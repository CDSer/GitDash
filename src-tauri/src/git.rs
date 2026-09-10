// Git 执行器模块
// 负责执行 Git 命令、解析状态、并发控制

use crate::models::{ChangedFile, GitResult, ProjectStatus};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::Semaphore;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Git 命令执行器
/// 支持并发控制和超时机制
pub struct GitExecutor {
    /// 并发信号量，限制同时执行的 Git 命令数量
    semaphore: Mutex<Arc<Semaphore>>,
    /// 自定义 Git 可执行文件路径
    git_path: Mutex<Option<String>>,
}

impl GitExecutor {
    /// 创建新的 Git 执行器
    ///
    /// # Arguments
    /// * `max_concurrent` - 最大并发数
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Mutex::new(Arc::new(Semaphore::new(max_concurrent.max(1)))),
            git_path: Mutex::new(None),
        }
    }

    /// 设置 Git 可执行文件路径（None 表示回退到 PATH 中的 git）
    pub fn set_git_path(&self, path: Option<String>) {
        let mut git_path = self.git_path.lock();
        *git_path = path;
    }

    /// 更新最大并发数（替换信号量；进行中的命令仍受旧限制约束）
    pub fn set_max_concurrent(&self, max_concurrent: usize) {
        let max_concurrent = max_concurrent.max(1);
        let mut sem = self.semaphore.lock();
        *sem = Arc::new(Semaphore::new(max_concurrent));
    }

    /// 执行 Git 命令
    /// 
    /// # Arguments
    /// * `repo_path` - 仓库路径
    /// * `args` - Git 命令参数
    /// 
    /// # Returns
    /// GitResult - 执行结果
    pub async fn exec(&self, repo_path: &str, args: &[&str]) -> GitResult {
        // 获取信号量许可，限制并发（克隆 Arc，避免跨 await 持锁）
        let semaphore = self.semaphore.lock().clone();
        let _permit = semaphore.acquire().await.unwrap();
        
        let start = Instant::now();
        let mut cmd = self.create_command(repo_path);
        
        for arg in args {
            cmd.arg(arg);
        }
        
        // 设置 120 秒超时（pull/push/fetch 等网络操作可能较慢）
        let timeout_duration = std::time::Duration::from_secs(120);
        let result = tokio::time::timeout(timeout_duration, cmd.output()).await;
        
        let result = match result {
            Ok(inner_result) => inner_result,
            Err(_) => {
                return GitResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "命令超时".to_string(),
                    duration_ms: 30000,
                };
            }
        };
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        match result {
            Ok(output) => GitResult {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms,
            },
            Err(e) => GitResult {
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
                duration_ms,
            },
        }
    }

    /// 获取项目 Git 状态
    /// 
    /// # Arguments
    /// * `repo_path` - 仓库路径
    /// 
    /// # Returns
    /// ProjectStatus - 项目状态
    pub async fn status(&self, repo_path: &str) -> ProjectStatus {
        let result = self.exec(repo_path, &["status", "--porcelain", "-b", "--untracked-files=all"]).await;
        
        if !result.success {
            return ProjectStatus {
                project_id: String::new(),
                branch: String::new(),
                ahead: 0,
                behind: 0,
                modified: 0,
                staged: 0,
                untracked: 0,
                is_clean: false,
                is_detached: false,
                changed_files: Vec::new(),
                last_fetched: None,
                is_fetching: false,
                error: Some(result.stderr),
            };
        }
        
        self.parse_status_output(&result.stdout)
    }

    /// 解析 git status 输出（porcelain v1，含逐文件改动与 detached 状态）
    fn parse_status_output(&self, output: &str) -> ProjectStatus {
        let mut branch = String::new();
        let mut ahead = 0u32;
        let mut behind = 0u32;
        let mut is_detached = false;
        let mut modified = 0u32;
        let mut staged = 0u32;
        let mut untracked = 0u32;
        let mut changed_files: Vec<ChangedFile> = Vec::new();

        let lines: Vec<&str> = output.lines().collect();

        if let Some(first_line) = lines.first() {
            if first_line.starts_with("## ") {
                // 分离 HEAD：## HEAD (no branch) 或 ## HEAD (no branch, ...)
                let rest = &first_line[3..];
                is_detached = rest.contains("no branch") || rest.trim_start().starts_with("HEAD (");
                branch = self.parse_branch_line(first_line);
                let (a, b) = self.parse_ahead_behind(first_line);
                ahead = a;
                behind = b;
            }
        }

        // 跳过第一行的分支信息
        for line in lines.iter().skip(1) {
            if line.is_empty() || line.len() < 3 {
                continue;
            }

            let index_status = &line[0..1];
            let worktree_status = &line[1..2];
            let rest = &line[3..]; // 跳过 "XY "

            // 改名 / 拷贝：porcelain 可能形如 "old -> new"（部分版本用 tab 分隔 "old\tnew"）
            let (path, original_path) = if index_status.starts_with('R')
                || index_status.starts_with('C')
                || worktree_status.starts_with('R')
                || worktree_status.starts_with('C')
            {
                let (old, new): (&str, &str) = if rest.contains('\t') {
                    let parts: Vec<&str> = rest.split('\t').collect();
                    (parts.first().copied().unwrap_or(""), parts.get(1).copied().unwrap_or(""))
                } else if let Some(pos) = rest.find(" -> ") {
                    let (o, n) = rest.split_at(pos);
                    (o.trim(), n[4..].trim()) // " -> " 长度为 4
                } else {
                    ("", rest)
                };
                if new.is_empty() {
                    (rest.to_string(), None)
                } else {
                    (new.to_string(), Some(old.to_string()))
                }
            } else {
                (rest.to_string(), None)
            };

            let is_staged = index_status != " " && index_status != "?";
            if is_staged {
                staged += 1;
            }
            if worktree_status != " " && worktree_status != "?" && worktree_status != "." {
                modified += 1;
            }
            if worktree_status == "?" || index_status == "?" {
                untracked += 1;
            }

            changed_files.push(ChangedFile {
                path,
                original_path,
                index_status: index_status.to_string(),
                worktree_status: worktree_status.to_string(),
                staged: is_staged,
            });
        }

        let is_clean = modified == 0 && staged == 0 && untracked == 0;

        ProjectStatus {
            project_id: String::new(),
            branch,
            ahead,
            behind,
            modified,
            staged,
            untracked,
            is_clean,
            is_detached,
            changed_files,
            last_fetched: None,
            is_fetching: false,
            error: None,
        }
    }

    /// 解析分支行
    fn parse_branch_line(&self, line: &str) -> String {
        // ## main...origin/main [ahead 2, behind 1]
        // 或 ## main (no remote)
        // 或 ## HEAD (no branch)
        
        let parts: Vec<&str> = line.split("...").collect();
        if parts.is_empty() {
            return String::new();
        }
        
        let branch_part = parts[0].trim_start_matches("## ");
        
        // 移除尾部空格和括号内容
        if let Some(idx) = branch_part.find(' ') {
            branch_part[..idx].to_string()
        } else {
            branch_part.to_string()
        }
    }

    /// 解析 ahead/behind 数量
    fn parse_ahead_behind(&self, line: &str) -> (u32, u32) {
        let mut ahead = 0u32;
        let mut behind = 0u32;
        
        if let Some(start) = line.find("[ahead ") {
            if let Some(end) = line[start..].find(']') {
                let bracket_content = &line[start + 1..start + end];
                
                if let Some(ahead_start) = bracket_content.find("ahead ") {
                    let ahead_str = &bracket_content[ahead_start + 6..];
                    if let Some(comma) = ahead_str.find(',') {
                        ahead = ahead_str[..comma].trim().parse().unwrap_or(0);
                    } else {
                        ahead = ahead_str.trim().parse().unwrap_or(0);
                    }
                }
                
                if let Some(behind_start) = bracket_content.find("behind ") {
                    let behind_str = &bracket_content[behind_start + 7..];
                    behind = behind_str.trim().trim_end_matches(']').parse().unwrap_or(0);
                }
            }
        }
        
        (ahead, behind)
    }

    /// 创建 Git 命令
    fn create_command(&self, repo_path: &str) -> Command {
        let git_path = self.git_path.lock();
        let git_cmd = git_path.as_deref().unwrap_or("git");

        let mut cmd = Command::new(git_cmd);
        cmd.current_dir(repo_path);

        // Windows 上避免每次执行 git 都闪现控制台窗口
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        // 禁用路径中的非 ASCII 字符被转义为 \xxx 八进制，确保 status/diff 输出原始 UTF-8 路径
        cmd.arg("-c").arg("core.quotepath=off");
        // 禁止交互式提示
        cmd.env("GIT_TERMINAL_PROMPT", "0");
        cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");

        cmd
    }
}
