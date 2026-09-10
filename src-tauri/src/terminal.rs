// 终端模块：基于 portable-pty 的交互式终端会话
// 每个项目一个会话；输出经 Tauri 事件流推送到前端（base64）

use crate::store::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use parking_lot::Mutex;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// 回滚缓冲上限（约 256KB）
const MAX_SCROLLBACK: usize = 256 * 1024;

#[derive(Clone, Serialize)]
struct TerminalDataEvent {
    #[serde(rename = "projectId")]
    project_id: String,
    /// base64 编码的终端输出字节
    data: String,
}

#[derive(Clone, Serialize)]
struct TerminalExitEvent {
    #[serde(rename = "projectId")]
    project_id: String,
}

struct Session {
    writer: Mutex<Box<dyn Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    /// 与读取线程共享的回滚缓冲
    scrollback: Arc<Mutex<Vec<u8>>>,
    alive: Arc<AtomicBool>,
}

pub struct TerminalManager {
    sessions: Mutex<HashMap<String, Arc<Session>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn close(&self, project_id: &str) {
        if let Some(session) = self.sessions.lock().remove(project_id) {
            session.alive.store(false, Ordering::SeqCst);
            let _ = session.killer.lock().kill();
        }
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        let sessions = self.sessions.lock();
        for session in sessions.values() {
            session.alive.store(false, Ordering::SeqCst);
            let _ = session.killer.lock().kill();
        }
    }
}

/// 默认用户 shell：Unix 取 $SHELL，Windows 优先 PowerShell
fn default_shell() -> (String, Vec<String>) {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        (shell, Vec::new())
    }
    #[cfg(windows)]
    {
        let ps = r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe";
        if std::path::Path::new(ps).exists() {
            (ps.to_string(), vec!["-NoLogo".to_string()])
        } else {
            let cmd = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
            (cmd, Vec::new())
        }
    }
}

fn append_scrollback(buf: &mut Vec<u8>, chunk: &[u8]) {
    buf.extend_from_slice(chunk);
    if buf.len() > MAX_SCROLLBACK {
        let excess = buf.len() - MAX_SCROLLBACK;
        buf.drain(..excess);
    }
}

/// 打开（或复用）项目终端；返回可回放的历史输出（base64）
#[tauri::command]
pub fn terminal_open(
    app: AppHandle,
    state: State<'_, AppState>,
    tm: State<'_, TerminalManager>,
    project_id: String,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    let cwd = {
        let config = state.config.read();
        config
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
            .ok_or_else(|| "项目不存在".to_string())?
    };

    // 已有存活会话：直接复用并回放
    {
        let sessions = tm.sessions.lock();
        if let Some(session) = sessions.get(&project_id) {
            if session.alive.load(Ordering::SeqCst) {
                let _ = session.master.lock().resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
                return Ok(STANDARD.encode(session.scrollback.lock().as_slice()));
            }
        }
    }
    // 清理死会话
    tm.sessions.lock().remove(&project_id);

    let pair = native_pty_system()
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("创建伪终端失败：{}", e))?;

    let (shell, args) = default_shell();
    let mut cmd = CommandBuilder::new(shell);
    for a in args {
        cmd.arg(a);
    }
    cmd.cwd(&cwd);
    // 继承应用进程完整环境（PATH、nvm 等）
    for (k, v) in std::env::vars() {
        cmd.env(k, v);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("启动 shell 失败：{}", e))?;
    let killer = child.clone_killer();
    // Unix：丢弃 slave，子进程退出时 reader 才能读到 EOF
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("创建读取器失败：{}", e))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("创建写入器失败：{}", e))?;

    let scrollback = Arc::new(Mutex::new(Vec::new()));
    let alive = Arc::new(AtomicBool::new(true));

    let session = Arc::new(Session {
        writer: Mutex::new(writer),
        master: Mutex::new(pair.master),
        killer: Mutex::new(killer),
        scrollback: Arc::clone(&scrollback),
        alive: Arc::clone(&alive),
    });

    // 输出读取线程：追加 scrollback 并向前端推送
    {
        let sb = Arc::clone(&scrollback);
        let alive = Arc::clone(&alive);
        let project_id = project_id.clone();
        let app = app.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        append_scrollback(&mut sb.lock(), &buf[..n]);
                        let _ = app.emit(
                            "terminal:data",
                            TerminalDataEvent {
                                project_id: project_id.clone(),
                                data: STANDARD.encode(&buf[..n]),
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
            alive.store(false, Ordering::SeqCst);
            let _ = app.emit(
                "terminal:exit",
                TerminalExitEvent {
                    project_id: project_id.clone(),
                },
            );
        });
    }

    tm.sessions.lock().insert(project_id, session);

    Ok(String::new())
}

/// 向终端写入用户输入（UTF-8 字符串）
#[tauri::command]
pub fn terminal_write(
    tm: State<'_, TerminalManager>,
    project_id: String,
    data: String,
) -> Result<(), String> {
    let sessions = tm.sessions.lock();
    let session = sessions
        .get(&project_id)
        .ok_or_else(|| "终端会话不存在".to_string())?;
    if !session.alive.load(Ordering::SeqCst) {
        return Err("终端会话已退出".to_string());
    }
    let mut writer = session.writer.lock();
    writer
        .write_all(data.as_bytes())
        .map_err(|e| format!("写入终端失败：{}", e))?;
    writer
        .flush()
        .map_err(|e| format!("刷新终端输出失败：{}", e))?;
    Ok(())
}

/// 调整终端尺寸
#[tauri::command]
pub fn terminal_resize(
    tm: State<'_, TerminalManager>,
    project_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let sessions = tm.sessions.lock();
    let session = sessions
        .get(&project_id)
        .ok_or_else(|| "终端会话不存在".to_string())?;
    session
        .master
        .lock()
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("调整终端尺寸失败：{}", e))?;
    Ok(())
}

/// 关闭终端会话（杀掉子进程）
#[tauri::command]
pub fn terminal_close(
    tm: State<'_, TerminalManager>,
    project_id: String,
) -> Result<(), String> {
    tm.close(&project_id);
    Ok(())
}
