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

/// Git 分支信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Branch {
    pub name: String,
    pub display_name: String,
    pub is_local: bool,
    pub is_remote: bool,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Git 提交记录
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Commit {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub date: i64,
    pub parents: Vec<String>,
}

/// 提交中的文件改动
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitFile {
    pub status: String,
    pub path: String,
}

/// 提交详情（含完整 message 和改动文件列表）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitDetail {
    #[serde(flatten)]
    pub commit: Commit,
    pub body: String,
    pub files: Vec<CommitFile>,
}

/// 文件树节点（单层，供前端懒加载文件树使用）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub has_children: bool,
}
