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
    #[serde(default = "default_scan_blacklist")]
    pub scan_blacklist: Vec<String>,
}

pub fn default_scan_blacklist() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".next".to_string(),
        ".nuxt".to_string(),
        "vendor".to_string(),
        "__pycache__".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
    ]
}

/// 单个文件的改动（状态详情用）
#[derive(Serialize, Clone, Debug, Default)]
pub struct ChangedFile {
    /// 仓库内相对路径
    pub path: String,
    /// 改名/拷贝前的原始路径（R/C 状态时有值）
    pub original_path: Option<String>,
    /// 暂存区状态（X 列）
    pub index_status: String,
    /// 工作区状态（Y 列）
    pub worktree_status: String,
    /// 是否已暂存（X 列非空且无改动标记）
    pub staged: bool,
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
    /// 是否处于分离 HEAD（detached）状态
    pub is_detached: bool,
    /// 逐文件改动列表（staging 面板用）
    pub changed_files: Vec<ChangedFile>,
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
    /// 关联项目 ID，前端据此匹配任务行
    pub project_id: String,
    pub status: String,
    pub message: Option<String>,
}

/// 带项目 ID 的 Git 执行结果（批量操作返回，避免下标错位）
#[derive(Serialize, Clone, Debug)]
pub struct ProjectGitResult {
    pub project_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
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
    /// 是否处于分离 HEAD
    pub is_detached: bool,
    /// 若来自 worktree，则记录其工作树路径
    pub worktree_path: Option<String>,
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

/// 提交中的文件改动（含增删行数、改名信息）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitFile {
    pub status: String,
    pub path: String,
    /// 改名/拷贝前的原路径
    pub original_path: Option<String>,
    /// 新增行数
    pub added: u32,
    /// 删除行数
    pub removed: u32,
    /// 是否为二进制文件
    pub is_binary: bool,
}

/// 提交结果（commit 命令返回）
#[derive(Serialize, Clone, Debug)]
pub struct GitCommitResult {
    pub commit_sha: String,
    pub summary: String,
}

/// 文件内容 diff 结果（左右对比用）
#[derive(Serialize, Clone, Debug)]
pub struct GitDiffContentResult {
    pub original_content: String,
    pub modified_content: String,
    pub is_binary: bool,
    /// 当无法做内容对比时的原始 patch 回退
    pub fallback_patch: String,
}

/// 丢弃改动条目（discard 命令入参）
#[derive(Deserialize, Clone, Debug)]
pub struct DiscardEntry {
    pub path: String,
    /// true=未跟踪文件（用 clean 删除），false=已跟踪文件（用 restore 还原）
    pub untracked: bool,
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

/// 批量扫描选项
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ScanOptions {
    #[serde(default)]
    pub max_depth: u32,
}

/// 扫描发现的候选仓库
#[derive(Serialize, Clone, Debug)]
pub struct ScannedRepo {
    pub path: String,
    pub name: String,
}

/// 批量导入结果
#[derive(Serialize, Clone, Debug)]
pub struct BatchImportResult {
    pub added: Vec<Project>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
}

/// 批量导入进度事件
#[derive(Serialize, Clone, Debug)]
pub struct ImportProgressEvent {
    pub scanned: u64,
    pub found: u64,
    pub current: String,
}
