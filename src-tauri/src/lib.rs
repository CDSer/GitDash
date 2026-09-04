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
            $crate::commands::add_project,
            $crate::commands::remove_project,
            $crate::commands::update_projects,
            $crate::commands::add_group,
            $crate::commands::remove_group,
            $crate::commands::update_groups,
            $crate::commands::get_project_status,
            $crate::commands::batch_pull,
            $crate::commands::batch_fetch,
            $crate::commands::batch_push,
            $crate::commands::open_repo_folder
        ]
    };
}
