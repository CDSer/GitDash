// GitDash 类型定义
// 这些类型与 Rust 后端的数据模型保持同步

export interface Project {
  id: string;
  name: string;
  path: string;
  group_id: string | null;
  tags: string[];
  is_favorite: boolean;
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
  last_fetched: number | null;
  is_fetching: boolean;
  error: string | null;
}

export interface GitResult {
  success: boolean;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

export interface OperationEvent {
  task_id: string;
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

/** Git 分支 */
export interface Branch {
  name: string;
  display_name: string;
  is_local: boolean;
  is_remote: boolean;
  is_current: boolean;
  upstream?: string;
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
}

/** 提交中的文件改动 */
export interface CommitFile {
  status: string;
  path: string;
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
