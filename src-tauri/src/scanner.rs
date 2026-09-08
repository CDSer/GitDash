// 项目扫描器模块
// 用于递归扫描目录中的 Git 仓库

use crate::models::{Project, ScanOptions, ScannedRepo};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

/// 默认扫描深度
const DEFAULT_MAX_DEPTH: u32 = 5;

/// 项目扫描器
pub struct ProjectScanner;

impl ProjectScanner {
    /// 扫描目录中的 Git 仓库
    pub fn scan_directory(
        base_path: &str,
        options: &ScanOptions,
        blacklist: &[String],
        app_handle: &AppHandle,
    ) -> Vec<ScannedRepo> {
        let max_depth = if options.max_depth == 0 {
            DEFAULT_MAX_DEPTH
        } else {
            options.max_depth
        };

        let base = PathBuf::from(base_path);
        let normalized_base = Self::normalize_path(&base);
        let mut repos = Vec::new();
        let mut scanned = 0u64;
        let mut found = 0u64;
        let mut last_emit = std::time::Instant::now();

        Self::scan_recursive(
            &base,
            &normalized_base,
            max_depth,
            blacklist,
            app_handle,
            &mut repos,
            &mut scanned,
            &mut found,
            &mut last_emit,
        );

        repos
    }

    /// 递归扫描实现
    #[allow(clippy::too_many_arguments)]
    fn scan_recursive(
        path: &Path,
        base_path: &str,
        depth: u32,
        blacklist: &[String],
        app_handle: &AppHandle,
        repos: &mut Vec<ScannedRepo>,
        scanned: &mut u64,
        found: &mut u64,
        last_emit: &mut std::time::Instant,
    ) {
        if depth == 0 {
            return;
        }

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }

            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if Self::is_blacklisted(&name, blacklist) {
                continue;
            }

            *scanned += 1;

            if Self::is_git_repository(&entry_path) {
                *found += 1;
                repos.push(ScannedRepo {
                    path: Self::normalize_path(&entry_path),
                    name,
                });
                // 仓库内部不再继续扫描
                continue;
            }

            Self::maybe_emit_progress(*scanned, *found, &entry_path, base_path, app_handle, last_emit);

            Self::scan_recursive(
                &entry_path,
                base_path,
                depth - 1,
                blacklist,
                app_handle,
                repos,
                scanned,
                found,
                last_emit,
            );
        }
    }

    /// 判断路径是否为 Git 仓库
    /// 支持普通仓库（.git 目录）和 git worktree（.git 文件指向真实 gitdir）
    fn is_git_repository(path: &Path) -> bool {
        let git_marker = path.join(".git");
        if !git_marker.exists() {
            return false;
        }

        if git_marker.is_dir() {
            return true;
        }

        // .git 是文件：可能是 worktree，读取指向的 gitdir
        if git_marker.is_file() {
            if let Ok(content) = fs::read_to_string(&git_marker) {
                let content = content.trim();
                if let Some(gitdir) = content.strip_prefix("gitdir: ") {
                    let resolved = path.join(gitdir.trim());
                    return resolved.is_dir();
                }
            }
        }

        false
    }

    /// 判断目录名是否在黑名单中
    fn is_blacklisted(name: &str, blacklist: &[String]) -> bool {
        blacklist.iter().any(|item| item == name)
    }

    /// 规范化路径（统一分隔符，去除末尾斜杠）
    fn normalize_path(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    /// 发送进度事件（节流，每 100ms 最多一次）
    fn maybe_emit_progress(
        scanned: u64,
        found: u64,
        current: &Path,
        base_path: &str,
        app_handle: &AppHandle,
        last_emit: &mut std::time::Instant,
    ) {
        if last_emit.elapsed().as_millis() < 100 {
            return;
        }

        let current_str = current
            .strip_prefix(base_path)
            .unwrap_or(current)
            .to_string_lossy()
            .to_string();

        let _ = app_handle.emit(
            "import:progress",
            crate::models::ImportProgressEvent {
                scanned,
                found,
                current: current_str,
            },
        );

        *last_emit = std::time::Instant::now();
    }

    /// 验证是否为有效的 Git 仓库
    pub fn is_valid_git_repo(path: &str) -> bool {
        Self::is_git_repository(Path::new(path))
    }

    /// 创建项目对象
    pub fn create_project(path: &Path, group_id: Option<String>) -> Option<Project> {
        let name = path.file_name()?.to_str()?.to_string();
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().timestamp();

        Some(Project {
            id,
            name,
            path: Self::normalize_path(path),
            group_id,
            tags: Vec::new(),
            created_at,
        })
    }

    /// 规范化路径集合（用于去重）
    pub fn normalize_for_dedup(path: &str) -> String {
        PathBuf::from(path).to_string_lossy().to_string()
    }

    /// 从现有项目中提取已存在的路径集合
    pub fn existing_paths(projects: &[Project]) -> HashSet<String> {
        projects
            .iter()
            .map(|p| Self::normalize_for_dedup(&p.path))
            .collect()
    }
}
