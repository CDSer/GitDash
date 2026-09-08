// GitDash 库模块
// 导出所有子模块

pub mod commands;
pub mod git;
pub mod models;
pub mod scanner;
pub mod store;
pub mod watcher;

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
            $crate::commands::git_diff,
            $crate::commands::git_diff_content,
            $crate::commands::git_show_commit,
            $crate::commands::git_commit_files,
            $crate::commands::git_commit_file_diff,
            $crate::commands::git_checkout_branch,
            $crate::commands::git_remote_url,
            $crate::commands::toggle_devtools
        ]
    };
}
