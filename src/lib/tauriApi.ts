// Tauri API 封装
// 封装所有与 Rust 后端的 IPC 调用

import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  Project,
  Group,
  ProjectGitResult,
  ProjectStatus,
  Branch,
  Commit,
  CommitDetail,
  CommitFile,
  ConflictFileContent,
  ConflictSide,
  FileNode,
  InProgressOp,
  MergeResult,
  Settings,
  GitCommitResult,
  GitDiffContentResult,
  DiscardEntry,
  ScanOptions,
  ScannedRepo,
  BatchImportResult,
} from '../types';

/**
 * 获取完整配置（项目、分组、设置）
 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

/**
 * 更新应用设置（Git 路径、自动 Fetch 间隔、并发数、主题、全局快捷键）
 * @param settings 设置对象
 */
export async function updateSettings(settings: Settings): Promise<void> {
  return invoke('update_settings', { settings });
}

/**
 * 添加新项目
 * @param path Git 仓库路径
 * @param groupId 分组 ID，为空则加入未分组
 */
export async function addProject(
  path: string,
  groupId?: string | null
): Promise<Project> {
  return invoke<Project>('add_project', { path, groupId: groupId ?? null });
}

/**
 * 扫描目录中的 Git 仓库候选
 * @param basePath Workspace 根目录
 * @param options 扫描选项
 */
export async function scanProjects(
  basePath: string,
  options?: ScanOptions
): Promise<ScannedRepo[]> {
  return invoke<ScannedRepo[]>('scan_projects', { basePath, options });
}

/**
 * 批量导入 Git 项目
 * @param paths 用户勾选的仓库路径列表
 * @param groupId 分组 ID，为空则加入未分组
 */
export async function batchImportProjects(
  paths: string[],
  groupId?: string | null
): Promise<BatchImportResult> {
  return invoke<BatchImportResult>('batch_import_projects', { paths, groupId: groupId ?? null });
}

/**
 * 删除项目
 * @param projectId 项目 ID
 */
export async function removeProject(projectId: string): Promise<void> {
  return invoke('remove_project', { projectId });
}

/**
 * 批量删除项目
 * @param projectIds 项目 ID 列表
 */
export async function removeProjects(projectIds: string[]): Promise<void> {
  return invoke('remove_projects', { projectIds });
}

/**
 * 批量更新项目信息
 * @param projects 项目列表
 */
export async function updateProjects(projects: Project[]): Promise<void> {
  return invoke('update_projects', { projects });
}

/**
 * 添加分组
 * @param name 分组名称
 * @param color 分组颜色
 */
export async function addGroup(name: string, color: string): Promise<Group> {
  return invoke<Group>('add_group', { name, color });
}

/**
 * 删除分组（该分组下的项目会变为未分组）
 * @param groupId 分组 ID
 */
export async function removeGroup(groupId: string): Promise<void> {
  return invoke('remove_group', { groupId });
}

/**
 * 批量更新分组信息
 * @param groups 分组列表
 */
export async function updateGroups(groups: Group[]): Promise<void> {
  return invoke('update_groups', { groups });
}

/**
 * 获取项目状态（分支、ahead/behind、修改文件数等）
 * @param projectId 项目 ID
 * @param force 是否强制刷新（忽略缓存）
 */
export async function getProjectStatus(
  projectId: string,
  force: boolean = false
): Promise<ProjectStatus> {
  return invoke<ProjectStatus>('get_project_status', { projectId, force });
}

/**
 * 批量 Pull 操作
 * @param projectIds 项目 ID 列表
 */
export async function batchPull(projectIds: string[]): Promise<ProjectGitResult[]> {
  return invoke<ProjectGitResult[]>('batch_pull', { projectIds });
}

/**
 * 批量 Fetch 操作
 * @param projectIds 项目 ID 列表
 */
export async function batchFetch(projectIds: string[]): Promise<ProjectGitResult[]> {
  return invoke<ProjectGitResult[]>('batch_fetch', { projectIds });
}

/**
 * 批量 Push 操作
 * @param projectIds 项目 ID 列表
 */
export async function batchPush(projectIds: string[]): Promise<ProjectGitResult[]> {
  return invoke<ProjectGitResult[]>('batch_push', { projectIds });
}

/**
 * 获取仓库分支列表
 * @param projectId 项目 ID
 */
export async function getBranches(projectId: string): Promise<Branch[]> {
  return invoke<Branch[]>('get_branches', { projectId });
}

/**
 * 获取提交记录列表
 * @param projectId 项目 ID
 * @param branch 分支名
 * @param limit 单页最大条数
 * @param beforeSha 分页游标：上一页最后一条提交的 SHA，用于续拉更早的提交
 */
export async function getCommits(
  projectId: string,
  branch: string,
  limit: number = 100,
  beforeSha?: string
): Promise<Commit[]> {
  return invoke<Commit[]>('get_commits', { projectId, branch, limit, beforeSha });
}

/**
 * 获取单次提交详情（含改动文件列表）
 * @param projectId 项目 ID
 * @param commitId 提交 ID
 */
export async function getCommitDetail(
  projectId: string,
  commitId: string
): Promise<CommitDetail> {
  return invoke<CommitDetail>('get_commit_detail', { projectId, commitId });
}

/**
 * 用系统文件管理器打开仓库文件夹
 * @param projectId 项目 ID
 */
export async function openRepoFolder(projectId: string): Promise<void> {
  return invoke('open_repo_folder', { projectId });
}

/**
 * 暂存指定文件
 * @param projectId 项目 ID
 * @param paths 相对仓库根目录的文件路径列表
 */
export async function gitStage(projectId: string, paths: string[]): Promise<void> {
  return invoke('git_stage', { projectId, paths });
}

/**
 * 取消暂存指定文件
 */
export async function gitUnstage(projectId: string, paths: string[]): Promise<void> {
  return invoke('git_unstage', { projectId, paths });
}

/**
 * 丢弃改动（已跟踪 restore，未跟踪 clean）
 * @param entries 丢弃条目列表
 */
export async function gitDiscard(projectId: string, entries: DiscardEntry[]): Promise<void> {
  return invoke('git_discard', { projectId, entries });
}

/**
 * 提交（返回新提交 sha + 摘要）
 * @param message 提交信息
 */
export async function gitCommit(projectId: string, message: string): Promise<GitCommitResult> {
  return invoke<GitCommitResult>('git_commit', { projectId, message });
}

/**
 * 查询进行中的 merge / rebase / cherry-pick / revert
 */
export async function gitInProgress(projectId: string): Promise<InProgressOp | null> {
  return invoke<InProgressOp | null>('git_in_progress', { projectId });
}

/**
 * 发起合并
 * @param branch 目标分支名
 */
export async function gitMerge(projectId: string, branch: string): Promise<MergeResult> {
  return invoke<MergeResult>('git_merge', { projectId, branch });
}

/**
 * 中止进行中的 merge / rebase / cherry-pick / revert
 */
export async function gitAbortOperation(projectId: string): Promise<void> {
  return invoke('git_abort_operation', { projectId });
}

/**
 * 继续进行中的合并/变基（需已解决全部冲突）
 * @param message 可选提交信息（仅 merge 使用）
 */
export async function gitMergeContinue(
  projectId: string,
  message?: string
): Promise<GitCommitResult> {
  return invoke<GitCommitResult>('git_merge_continue', {
    projectId,
    message: message ?? null,
  });
}

/**
 * 采纳一侧解决冲突（ours / theirs）并自动 add
 */
export async function gitResolveConflict(
  projectId: string,
  paths: string[],
  side: ConflictSide
): Promise<void> {
  return invoke('git_resolve_conflict', { projectId, paths, side });
}

/**
 * 手动编辑后标记冲突已解决（git add）
 */
export async function gitMarkConflictResolved(
  projectId: string,
  paths: string[]
): Promise<void> {
  return invoke('git_mark_conflict_resolved', { projectId, paths });
}

/**
 * 读取冲突文件三路内容（base / ours / theirs）
 */
export async function gitConflictFileContent(
  projectId: string,
  path: string
): Promise<ConflictFileContent> {
  return invoke<ConflictFileContent>('git_conflict_file_content', { projectId, path });
}

/**
 * 获取 diff（原始 patch 文本）
 * @param path 可选，指定文件则只返回该文件 diff
 * @param staged 是否查看已暂存区 diff
 */
export async function gitDiff(
  projectId: string,
  path?: string,
  staged: boolean = false
): Promise<string> {
  return invoke<string>('git_diff', { projectId, path, staged });
}

/**
 * 获取单文件左右对比内容 diff
 */
export async function gitDiffContent(
  projectId: string,
  path: string,
  staged: boolean = false,
  originalPath?: string
): Promise<GitDiffContentResult> {
  return invoke<GitDiffContentResult>('git_diff_content', {
    projectId,
    path,
    staged,
    originalPath,
  });
}

/**
 * 获取单次提交的完整 patch（含 stat）
 */
export async function gitShowCommit(projectId: string, sha: string): Promise<string> {
  return invoke<string>('git_show_commit', { projectId, sha });
}

/**
 * 获取单次提交改动的文件列表（含增删行数与改名）
 */
export async function gitCommitFiles(projectId: string, sha: string): Promise<CommitFile[]> {
  return invoke<CommitFile[]>('git_commit_files', { projectId, sha });
}

/**
 * 获取单次提交中单个文件的左右对比内容 diff
 */
export async function gitCommitFileDiff(
  projectId: string,
  sha: string,
  path: string,
  originalPath?: string
): Promise<GitDiffContentResult> {
  return invoke<GitDiffContentResult>('git_commit_file_diff', {
    projectId,
    sha,
    path,
    originalPath,
  });
}

/**
 * 切换分支
 */
export async function gitCheckoutBranch(projectId: string, branch: string): Promise<void> {
  return invoke('git_checkout_branch', { projectId, branch });
}

/**
 * 获取远端地址（默认 origin）
 */
export async function gitRemoteUrl(projectId: string, name?: string): Promise<string | null> {
  return invoke<string | null>('git_remote_url', { projectId, name });
}

/**
 * 强制刷新项目状态（忽略缓存）
 */
export async function getProjectStatusForce(
  projectId: string
): Promise<ProjectStatus> {
  return invoke<ProjectStatus>('get_project_status', { projectId, force: true });
}

/**
 * 列出目录内容（单层，用于懒加载文件树）
 * @param path 目录绝对路径
 */
export async function listDirectory(path: string): Promise<FileNode[]> {
  return invoke<FileNode[]>('list_directory', { path });
}

/**
 * 读取文本文件内容
 * @param path 文件绝对路径
 */
export async function readFile(path: string): Promise<string> {
  return invoke<string>('read_file', { path });
}

/**
 * 写入文件内容
 * @param path 文件绝对路径
 * @param content 文件内容
 */
export async function writeFile(path: string, content: string): Promise<void> {
  return invoke('write_file', { path, content });
}
