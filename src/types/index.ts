// GitDash 类型定义
// 这些类型与 Rust 后端的数据模型保持同步

export interface Project {
  id: string;
  name: string;
  path: string;
  group_id: string | null;
  tags: string[];
  created_at: number;
}

export interface Group {
  id: string;
  name: string;
  color: string;
  sort_order: number;
}

export interface Settings {
  git_path: string | null;
  auto_fetch_interval: number;
  max_concurrent_git: number;
  theme: 'system' | 'light' | 'dark';
  global_shortcut: string;
  scan_blacklist: string[];
}

export interface AppConfig {
  version: string;
  projects: Project[];
  groups: Group[];
  settings: Settings;
}

export interface ProjectStatus {
  project_id: string;
  branch: string;
  ahead: number;
  behind: number;
  modified: number;
  staged: number;
  untracked: number;
  is_clean: boolean;
  is_detached: boolean;
  changed_files: ChangedFile[];
  /** 进行中的 merge / rebase / cherry-pick / revert */
  in_progress: InProgressOp | null;
  /** 未解决冲突文件数 */
  conflict_count: number;
  last_fetched: number | null;
  is_fetching: boolean;
  error: string | null;
}

/** 单文件改动（staging 面板用） */
export interface ChangedFile {
  path: string;
  original_path?: string | null;
  index_status: string;
  worktree_status: string;
  staged: boolean;
  /** 未合并冲突文件 */
  is_conflict?: boolean;
}

/** 进行中的 Git 操作 */
export interface InProgressOp {
  kind: 'merge' | 'rebase' | 'cherry-pick' | 'revert' | string;
  head_message: string | null;
}

/** 冲突文件三路内容 */
export interface ConflictFileContent {
  path: string;
  base: string;
  ours: string;
  theirs: string;
  exists_base: boolean;
  exists_ours: boolean;
  exists_theirs: boolean;
  is_binary: boolean;
}

/** 采纳一侧解决冲突 */
export type ConflictSide = 'ours' | 'theirs';

/** 发起合并 / 继续合并结果 */
export interface MergeResult {
  success: boolean;
  has_conflicts: boolean;
  message: string;
}

export interface GitResult {
  success: boolean;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

export interface OperationEvent {
  task_id: string;
  /** 关联项目 ID，用于匹配任务行 */
  project_id: string;
  status: string;
  message: string | null;
}

export interface OperationTask {
  id: string;
  projectId: string;
  projectName: string;
  operation: 'pull' | 'push' | 'fetch';
  status: 'pending' | 'running' | 'success' | 'error';
  message?: string;
  createdAt: number;
}

/** 带项目 ID 的批量 Git 结果 */
export interface ProjectGitResult {
  project_id: string;
  success: boolean;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

/** Git 分支 */
export interface Branch {
  name: string;
  display_name: string;
  is_local: boolean;
  is_remote: boolean;
  is_current: boolean;
  upstream?: string | null;
  is_detached: boolean;
  worktree_path?: string | null;
}

/** Git 提交记录 */
export interface Commit {
  id: string;
  short_id: string;
  message: string;
  author: string;
  email: string;
  date: number;
  parents: string[];
  /** 是否已推送到上游（无上游时为 true） */
  is_pushed?: boolean;
}

/** 提交中的文件改动（含增删行数与改名） */
export interface CommitFile {
  status: string;
  path: string;
  original_path?: string | null;
  added: number;
  removed: number;
  is_binary: boolean;
}

/** 提交结果（commit 命令返回） */
export interface GitCommitResult {
  commit_sha: string;
  summary: string;
}

/** 文件内容 diff 结果（左右对比） */
export interface GitDiffContentResult {
  original_content: string;
  modified_content: string;
  is_binary: boolean;
  fallback_patch: string;
}

/** 丢弃改动条目（discard 命令入参） */
export interface DiscardEntry {
  path: string;
  untracked: boolean;
}

/** 提交详情 */
export type CommitDetail = Commit & { body: string; files: CommitFile[] };

/** 文件树节点（单层） */
export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  has_children: boolean;
}

/** 批量扫描选项 */
export interface ScanOptions {
  max_depth?: number;
}

/** 扫描发现的候选仓库 */
export interface ScannedRepo {
  path: string;
  name: string;
}

/** 批量导入结果 */
export interface BatchImportResult {
  added: Project[];
  skipped: string[];
  failed: string[];
}

/** 批量导入进度事件 */
export interface ImportProgressEvent {
  scanned: number;
  found: number;
  current: string;
}
