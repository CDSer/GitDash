// GitDash 库模块
// 导出所有子模块

pub mod commands;
pub mod git;
pub mod models;
pub mod scanner;
pub mod store;
pub mod terminal;
pub mod watcher;
pub mod window_controls;

// 重新导出常用类型（避免歧义）
pub use models::*;

/// 生成 Tauri invoke handler（必须在库 crate 内展开，命令宏生成的辅助宏在此可见）
#[macro_export]
macro_rules! invoke_handler {
    () => {
        tauri::generate_handler![
            $crate::commands::get_config,
            $crate::commands::update_settings,
            $crate::commands::add_project,
            $crate::commands::scan_projects,
            $crate::commands::batch_import_projects,
            $crate::commands::remove_project,
            $crate::commands::remove_projects,
            $crate::commands::update_projects,
            $crate::commands::add_group,
            $crate::commands::remove_group,
            $crate::commands::update_groups,
            $crate::commands::get_project_status,
            $crate::commands::get_branches,
            $crate::commands::get_commits,
            $crate::commands::get_commit_detail,
            $crate::commands::list_directory,
            $crate::commands::read_file,
            $crate::commands::write_file,
            $crate::commands::batch_pull,
            $crate::commands::batch_fetch,
            $crate::commands::batch_push,
            $crate::commands::open_repo_folder,
            $crate::commands::git_stage,
            $crate::commands::git_unstage,
            $crate::commands::git_discard,
            $crate::commands::git_commit,
            $crate::commands::git_in_progress,
            $crate::commands::git_merge,
            $crate::commands::git_abort_operation,
            $crate::commands::git_merge_continue,
            $crate::commands::git_resolve_conflict,
            $crate::commands::git_mark_conflict_resolved,
            $crate::commands::git_conflict_file_content,
            $crate::commands::git_diff,
            $crate::commands::git_diff_content,
            $crate::commands::git_show_commit,
            $crate::commands::git_commit_files,
            $crate::commands::git_commit_file_diff,
            $crate::commands::git_checkout_branch,
            $crate::commands::git_remote_url,
            $crate::commands::list_user_skins,
            $crate::terminal::terminal_open,
            $crate::terminal::terminal_write,
            $crate::terminal::terminal_resize,
            $crate::terminal::terminal_close,
            $crate::commands::toggle_devtools,
            $crate::window_controls::set_macos_traffic_light_position
        ]
    };
}
