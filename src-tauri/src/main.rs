// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use gitdash_lib::git::GitExecutor;
use gitdash_lib::models::{default_scan_blacklist, AppConfig, Settings};
use gitdash_lib::store::{AppState, StatusCache};
use gitdash_lib::watcher::WatcherManager;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化状态：从磁盘加载配置（首次运行时写入默认配置）
            let state = app.state::<AppState>();

            if let Err(e) = state.load_config() {
                eprintln!("加载配置失败：{}", e);
            }

            // 设置系统托盘
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            #[cfg(debug_assertions)]
            let devtools_i = MenuItem::with_id(app, "devtools", "打开开发者工具", true, None::<&str>)?;
            #[cfg(debug_assertions)]
            let menu = Menu::with_items(app, &[&show_i, &devtools_i, &quit_i])?;
            #[cfg(not(debug_assertions))]
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
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
                    auto_fetch_interval: 30,
                    max_concurrent_git: 3,
                    theme: "system".to_string(),
                    global_shortcut: "CommandOrControl+Shift+G".to_string(),
                    scan_blacklist: default_scan_blacklist(),
                },
            })),
            git: Arc::new(GitExecutor::new(3)),
            cache: Arc::new(StatusCache::new(5)),
            operation_queue: Mutex::new(Vec::new()),
        })
        .manage(WatcherManager::new())
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
