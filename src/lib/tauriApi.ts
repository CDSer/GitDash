// Tauri API 封装
// 封装所有与 Rust 后端的 IPC 调用

import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  Project,
  Group,
  GitResult,
  ProjectStatus,
  Branch,
  Commit,
  CommitDetail,
  FileNode,
} from '../types';

/**
 * 获取完整配置（项目、分组、设置）
 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

/**
 * 添加新项目
 * @param path Git 仓库路径
 */
export async function addProject(path: string): Promise<Project> {
  return invoke<Project>('add_project', { path });
}

/**
 * 删除项目
 * @param projectId 项目 ID
 */
export async function removeProject(projectId: string): Promise<void> {
  return invoke('remove_project', { projectId });
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
export async function batchPull(projectIds: string[]): Promise<GitResult[]> {
  return invoke<GitResult[]>('batch_pull', { projectIds });
}

/**
 * 批量 Fetch 操作
 * @param projectIds 项目 ID 列表
 */
export async function batchFetch(projectIds: string[]): Promise<GitResult[]> {
  return invoke<GitResult[]>('batch_fetch', { projectIds });
}

/**
 * 批量 Push 操作
 * @param projectIds 项目 ID 列表
 */
export async function batchPush(projectIds: string[]): Promise<GitResult[]> {
  return invoke<GitResult[]>('batch_push', { projectIds });
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
