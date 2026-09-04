// 数据模型定义
// 所有数据结构必须与前端 TypeScript 类型保持一致

use serde::{Deserialize, Serialize};

/// 应用配置（包含所有项目、分组、设置）
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppConfig {
    pub version: String,
    pub projects: Vec<Project>,
    pub groups: Vec<Group>,
    pub settings: Settings,
}

/// Git 仓库项目
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub group_id: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub created_at: i64,
}

/// 项目分组
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i32,
}

/// 应用设置
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Settings {
    pub git_path: Option<String>,
    pub auto_fetch_interval: u64,
    pub max_concurrent_git: usize,
    pub theme: String,
    pub global_shortcut: String,
}

/// 项目 Git 状态
#[derive(Serialize, Clone, Debug, Default)]
pub struct ProjectStatus {
    pub project_id: String,
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub modified: u32,
    pub staged: u32,
    pub untracked: u32,
    pub is_clean: bool,
    pub last_fetched: Option<i64>,
    pub is_fetching: bool,
    pub error: Option<String>,
}

/// Git 命令执行结果
#[derive(Serialize, Clone, Debug)]
pub struct GitResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// 操作进度事件（用于前端实时显示）
#[derive(Serialize, Clone, Debug)]
pub struct OperationEvent {
    pub task_id: String,
    pub status: String,
    pub message: Option<String>,
}
