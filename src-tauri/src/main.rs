// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use gitdash_lib::commands::apply_runtime_settings;
use gitdash_lib::git::GitExecutor;
use gitdash_lib::models::{default_scan_blacklist, AppConfig, Settings};
use gitdash_lib::store::{AppState, StatusCache};
use gitdash_lib::terminal::TerminalManager;
use gitdash_lib::watcher::WatcherManager;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all())
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = shortcut; // 默认只绑定「显示窗口」
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 恢复上次窗口位置/尺寸/最大化状态
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.restore_state(StateFlags::all());
            }

            // 初始化状态：从磁盘加载配置（首次运行时写入默认配置）
            let state = app.state::<AppState>();

            if let Err(e) = state.load_config() {
                eprintln!("加载配置失败：{}", e);
            }

            // 把磁盘上的设置应用到运行时（Git 路径、并发数、全局快捷键）
            {
                let settings = state.config.read().settings.clone();
                apply_runtime_settings(app.handle(), &state, &settings);
            }

            // 后台按 auto_fetch_interval 定时 fetch 全部仓库（0 = 关闭）
            {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    auto_fetch_loop(app_handle).await;
                });
            }

            // 设置系统托盘
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let fetch_all_i =
                MenuItem::with_id(app, "fetch_all", "立即获取全部 (Fetch)", true, None::<&str>)?;
            #[cfg(debug_assertions)]
            let devtools_i = MenuItem::with_id(app, "devtools", "打开开发者工具", true, None::<&str>)?;
            #[cfg(debug_assertions)]
            let menu = Menu::with_items(app, &[&show_i, &fetch_all_i, &devtools_i, &quit_i])?;
            #[cfg(not(debug_assertions))]
            let menu = Menu::with_items(app, &[&show_i, &fetch_all_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("GitDash")
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        let _ = app.save_window_state(StateFlags::all());
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "fetch_all" => {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            fetch_all_projects(&handle).await;
                        });
                    }
                    "devtools" => {
                        gitdash_lib::commands::toggle_devtools(app.clone());
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(gitdash_lib::invoke_handler!())
        .manage(AppState {
            config: Arc::new(parking_lot::RwLock::new(AppConfig {
                version: "1.0.0".to_string(),
                projects: Vec::new(),
                groups: Vec::new(),
                settings: Settings {
                    git_path: None,
                    auto_fetch_interval: 600,
                    max_concurrent_git: 3,
                    theme: "system".to_string(),
                    skin: "default".to_string(),
                    global_shortcut: "CommandOrControl+Shift+G".to_string(),
                    scan_blacklist: default_scan_blacklist(),
                },
            })),
            git: Arc::new(GitExecutor::new(3)),
            cache: Arc::new(StatusCache::new(5)),
            operation_queue: Mutex::new(Vec::new()),
        })
        .manage(WatcherManager::new())
        .manage(TerminalManager::new())
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}

/// 定时对所有已添加仓库执行 `git fetch --prune --all`
/// `auto_fetch_interval` 为 0 时关闭自动获取
async fn auto_fetch_loop(app_handle: tauri::AppHandle) {
    loop {
        let (interval, project_paths) = {
            let state = app_handle.state::<AppState>();
            let config = state.config.read();
            (
                config.settings.auto_fetch_interval,
                config
                    .projects
                    .iter()
                    .map(|p| p.path.clone())
                    .collect::<Vec<_>>(),
            )
        };

        if interval == 0 {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            continue;
        }

        // 先 sleep，避免启动瞬间打满远端
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

        fetch_paths_now(&app_handle, &project_paths).await;
    }
}

/// 托盘「立即获取全部」：对当前全部项目 fetch
async fn fetch_all_projects(app_handle: &tauri::AppHandle) {
    let paths: Vec<String> = {
        let state = app_handle.state::<AppState>();
        let config = state.config.read();
        config.projects.iter().map(|p| p.path.clone()).collect()
    };
    fetch_paths_now(app_handle, &paths).await;
}

async fn fetch_paths_now(app_handle: &tauri::AppHandle, paths: &[String]) {
    let state = app_handle.state::<AppState>();
    for path in paths {
        let _ = state.git.exec(path, &["fetch", "--prune", "--all"]).await;
    }
}
