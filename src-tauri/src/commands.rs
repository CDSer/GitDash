// Tauri Commands 模块
// 定义所有暴露给前端的 Rust 命令

use crate::models::{
    AppConfig, Branch, Commit, CommitDetail, CommitFile, DiscardEntry, FileNode, GitCommitResult,
    GitDiffContentResult, GitResult, Group, OperationEvent, Project, ProjectStatus, Settings,
};
use crate::git::GitExecutor;
use crate::scanner::ProjectScanner;
use crate::store::{AppState, StatusCache};
use crate::watcher::WatcherManager;
use tauri::{webview::WebviewWindow, Emitter, Manager, State};
use uuid::Uuid;

/// 获取完整配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read();
    Ok(config.clone())
}

/// 更新应用设置（并持久化到配置文件）
#[tauri::command]
pub async fn update_settings(
    settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut config = state.config.write();
        config.settings = settings.clone();
        save_config(&config, &state.cache)?;
    }
    Ok(())
}

/// 添加新项目
#[tauri::command]
pub async fn add_project(
    path: String,
    state: State<'_, AppState>,
    watcher: State<'_, WatcherManager>,
    app_handle: tauri::AppHandle,
) -> Result<Project, String> {
    if !ProjectScanner::is_valid_git_repo(&path) {
        return Err("不是有效的 Git 仓库".to_string());
    }

    let name = std::path::Path::new(&path)
        .file_name()
        .ok_or("路径无效")?
        .to_str()
        .ok_or("路径编码无效")?
        .to_string();

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        path: path.clone(),
        group_id: None,
        tags: Vec::new(),
        created_at: chrono::Utc::now().timestamp(),
    };

    {
        let mut config = state.config.write();
        config.projects.push(project.clone());
        save_config(&config, &state.cache)?;
    }

    watcher.add_project(&project, &app_handle).await?;

    Ok(project)
}

/// 删除项目
#[tauri::command]
pub async fn remove_project(
    project_id: String,
    state: State<'_, AppState>,
    watcher: State<'_, WatcherManager>,
) -> Result<(), String> {
    {
        let mut config = state.config.write();
        config.projects.retain(|p| p.id != project_id);
        save_config(&config, &state.cache)?;
    }

    watcher.remove_project(&project_id).await;
    state.cache.invalidate(&project_id);

    Ok(())
}

/// 批量更新项目
#[tauri::command]
pub async fn update_projects(
    projects: Vec<Project>,
    state: State<'_, AppState>,
    watcher: State<'_, WatcherManager>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let updates = {
        let mut config = state.config.write();
        let mut updates = Vec::new();

        for updated_project in &projects {
            if let Some(existing) = config.projects.iter_mut().find(|p| p.id == updated_project.id) {
                updates.push((existing.clone(), updated_project.clone()));
                *existing = updated_project.clone();
            }
        }

        save_config(&config, &state.cache)?;
        updates
    };

    for (old_project, updated_project) in updates {
        watcher.update_project(&old_project, &updated_project, &app_handle).await?;
    }

    Ok(())
}

/// 添加分组
#[tauri::command]
pub async fn add_group(
    name: String,
    color: String,
    state: State<'_, AppState>,
) -> Result<Group, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("分组名称不能为空".to_string());
    }

    let mut config = state.config.write();

    if config.groups.iter().any(|g| g.name == name) {
        return Err("分组名称已存在".to_string());
    }

    let sort_order = config
        .groups
        .iter()
        .map(|g| g.sort_order)
        .max()
        .map(|v| v + 1)
        .unwrap_or(0);

    let group = Group {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        sort_order,
    };

    config.groups.push(group.clone());
    save_config(&config, &state.cache)?;

    Ok(group)
}

/// 删除分组（该分组下的项目变为未分组）
#[tauri::command]
pub async fn remove_group(
    group_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write();

    config.groups.retain(|g| g.id != group_id);

    for project in config.projects.iter_mut() {
        if project.group_id.as_deref() == Some(group_id.as_str()) {
            project.group_id = None;
        }
    }

    save_config(&config, &state.cache)?;

    Ok(())
}

/// 批量更新分组（重命名、改色、排序）
#[tauri::command]
pub async fn update_groups(
    groups: Vec<Group>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write();

    for updated_group in &groups {
        if let Some(existing) = config.groups.iter_mut().find(|g| g.id == updated_group.id) {
            *existing = updated_group.clone();
        }
    }

    save_config(&config, &state.cache)?;

    Ok(())
}

/// 获取项目状态
#[tauri::command]
pub async fn get_project_status(
    project_id: String,
    force: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ProjectStatus, String> {
    let force = force.unwrap_or(false);
    
    if !force {
        if let Some(cached) = state.cache.get(&project_id) {
            return Ok(cached);
        }
    }

    let project = {
        let config = state.config.read();
        config.projects.iter()
            .find(|p| p.id == project_id)
            .ok_or("未找到项目")?
            .clone()
    };

    let mut status = state.git.status(&project.path).await;
    status.project_id = project_id.clone();

    if status.is_clean {
        state.cache.set(project_id.clone(), status.clone());
    }

    Ok(status)
}

/// 获取仓库分支列表（含 detached HEAD 与 worktree 信息）
#[tauri::command]
pub async fn get_branches(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Branch>, String> {
    let project = {
        let config = state.config.read();
        config.projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
    };

    let project = project.ok_or("未找到项目")?;

    let current_result = state
        .git
        .exec(&project.path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .await;
    let current_branch = if current_result.success {
        current_result.stdout.trim().to_string()
    } else {
        String::new()
    };
    let is_detached_head = current_branch == "HEAD";

    let result = state
        .git
        .exec(
            &project.path,
            &[
                "for-each-ref",
                "--format=%(refname)\t%(upstream:short)",
                "refs/heads",
                "refs/remotes",
            ],
        )
        .await;

    if !result.success {
        return Err(format!("获取分支失败：{}", result.stderr));
    }

    let mut branches: Vec<Branch> = Vec::new();

    for line in result.stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let refname = parts[0];
        let upstream = if parts[1].is_empty() {
            None
        } else {
            Some(parts[1].to_string())
        };

        let (full_name, display_name, is_local, is_remote) =
            if let Some(name) = refname.strip_prefix("refs/heads/") {
                (name.to_string(), name.to_string(), true, false)
            } else if let Some(name) = refname.strip_prefix("refs/remotes/") {
                (name.to_string(), name.to_string(), false, true)
            } else {
                continue;
            };

        let is_current = is_local && current_branch == display_name;

        branches.push(Branch {
            name: full_name,
            display_name,
            is_local,
            is_remote,
            is_current,
            upstream,
            is_detached: false,
            worktree_path: None,
        });
    }

    // 分离 HEAD：补充一个当前条目
    if is_detached_head {
        let sha_res = state.git.exec(&project.path, &["rev-parse", "HEAD"]).await;
        let short = sha_res.stdout.trim();
        let short7 = if short.len() >= 7 { &short[..7] } else { short };
        branches.push(Branch {
            name: format!("(detached @ {})", short7),
            display_name: format!("(detached @ {})", short7),
            is_local: true,
            is_remote: false,
            is_current: true,
            upstream: None,
            is_detached: true,
            worktree_path: None,
        });
    }

    // worktree 列表（同一分支在其它工作树中检出时标注路径）
    let wt = state
        .git
        .exec(&project.path, &["worktree", "list", "--porcelain"])
        .await;
    if wt.success {
        let mut current_path: Option<String> = None;
        let mut wt_branch: Option<String> = None;
        let mut wt_bare = false;
        let mut wt_head_sha: Option<String> = None;
        for line in wt.stdout.lines() {
            if let Some(rest) = line.strip_prefix("worktree ") {
                if let Some(p) = current_path.take() {
                    if !wt_bare {
                        push_worktree(&mut branches, p, wt_branch.take(), wt_head_sha.take());
                    }
                }
                current_path = Some(rest.trim().to_string());
                wt_bare = false;
                wt_head_sha = None;
            } else if let Some(rest) = line.strip_prefix("HEAD ") {
                wt_head_sha = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("branch ") {
                let raw = rest.trim();
                wt_branch = Some(raw.strip_prefix("refs/heads/").unwrap_or(raw).to_string());
            } else if line.starts_with("bare") {
                wt_bare = true;
            }
        }
        if let Some(p) = current_path.take() {
            if !wt_bare {
                push_worktree(&mut branches, p, wt_branch.take(), wt_head_sha.take());
            }
        }
    }

    // 去重：worktree 条目优先覆盖同名 local 的 worktree_path
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut deduped: Vec<Branch> = Vec::with_capacity(branches.len());
    for b in branches {
        if let Some(&i) = seen.get(&b.name) {
            let existing = &mut deduped[i];
            if b.worktree_path.is_some() && existing.worktree_path.is_none() && !existing.is_current {
                existing.worktree_path = b.worktree_path;
            } else if b.is_current && !existing.is_current {
                existing.is_current = true;
            }
        } else {
            seen.insert(b.name.clone(), deduped.len());
            deduped.push(b);
        }
    }

    Ok(deduped)
}

/// 将一个 worktree 分支条目追加到分支列表
fn push_worktree(branches: &mut Vec<Branch>, path: String, branch: Option<String>, _head_sha: Option<String>) {
    let name = match branch {
        Some(b) => b,
        None => return,
    };
    branches.push(Branch {
        name: name.clone(),
        display_name: name,
        is_local: true,
        is_remote: false,
        is_current: false,
        upstream: None,
        is_detached: false,
        worktree_path: Some(path),
    });
}

/// 获取提交记录列表
/// 支持 before_sha 游标：传入上一页最后一条提交的 SHA 时，
/// 使用 `git log <sha>^` 续拉更早的提交（分页加载）。
#[tauri::command]
pub async fn get_commits(
    project_id: String,
    branch: String,
    limit: Option<usize>,
    before_sha: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Commit>, String> {
    let project = {
        let config = state.config.read();
        config.projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
    };

    let project = project.ok_or("未找到项目")?;
    let limit = limit.unwrap_or(100);

    let mut owned: Vec<String> = Vec::new();
    owned.push("log".to_string());
    owned.push(format!("--max-count={}", limit));
    owned.push(
        "--format=%H%x09%h%x09%s%x09%an%x09%ae%x09%at%x09%P".to_string(),
    );
    match &before_sha {
        Some(sha) if !sha.is_empty() => {
            if !is_safe_sha(sha) {
                return Err("无效的游标 SHA".to_string());
            }
            // 从给定提交的第一个父节点继续（排除该提交本身）
            owned.push(format!("{}^", sha));
        }
        _ => {
            owned.push(branch.clone());
        }
    }
    let args: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();

    let result = state.git.exec(&project.path, &args).await;

    if !result.success {
        return Err(format!("获取提交记录失败：{}", result.stderr));
    }

    let mut commits = Vec::new();

    for line in result.stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 6 {
            continue;
        }

        let id = parts[0].to_string();
        let short_id = parts[1].to_string();
        let message = parts[2].to_string();
        let author = parts[3].to_string();
        let email = parts[4].to_string();
        let date = parts[5].parse::<i64>().unwrap_or(0);
        let parents: Vec<String> = if parts.len() > 6 && !parts[6].is_empty() {
            parts[6].split_whitespace().map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        };

        commits.push(Commit {
            id,
            short_id,
            message,
            author,
            email,
            date,
            parents,
        });
    }

    Ok(commits)
}

/// 获取单次提交详情（含完整 message 和改动文件列表）
#[tauri::command]
pub async fn get_commit_detail(
    project_id: String,
    commit_id: String,
    state: State<'_, AppState>,
) -> Result<CommitDetail, String> {
    let project = {
        let config = state.config.read();
        config.projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
    };

    let project = project.ok_or("未找到项目")?;

    let result = state.git.exec(
        &project.path,
        &[
            "show",
            "-s",
            "--format=%H%x09%h%x09%s%x09%an%x09%ae%x09%at%x09%P%n%b",
            &commit_id,
        ],
    ).await;

    if !result.success {
        return Err(format!("获取提交详情失败：{}", result.stderr));
    }

    let stdout = result.stdout;
    let (first_line, body) = if let Some(idx) = stdout.find('\n') {
        (stdout[..idx].trim(), stdout[idx + 1..].trim().to_string())
    } else {
        (stdout.trim(), String::new())
    };

    let parts: Vec<&str> = first_line.split('\t').collect();
    if parts.len() < 6 {
        return Err("提交详情格式异常".to_string());
    }

    let id = parts[0].to_string();
    let short_id = parts[1].to_string();
    let message = parts[2].to_string();
    let author = parts[3].to_string();
    let email = parts[4].to_string();
    let date = parts[5].parse::<i64>().unwrap_or(0);
    let parents: Vec<String> = if parts.len() > 6 && !parts[6].is_empty() {
        parts[6].split_whitespace().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    let files = load_commit_files(&state.git, &project.path, &commit_id).await;

    Ok(CommitDetail {
        commit: Commit {
            id,
            short_id,
            message,
            author,
            email,
            date,
            parents,
        },
        body,
        files,
    })
}

/// 批量 Pull
#[tauri::command]
pub async fn batch_pull(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<Vec<GitResult>, String> {
    let projects: Vec<_> = {
        let config = state.config.read();
        config.projects
            .iter()
            .filter(|p| project_ids.contains(&p.id))
            .cloned()
            .collect()
    };

    let mut results = Vec::new();

    for project in &projects {
        let task_id = Uuid::new_v4().to_string();
        
        let _ = window.emit("git:progress", OperationEvent {
            task_id: task_id.clone(),
            status: "running".to_string(),
            message: Some(format!("正在拉取 {}...", project.name)),
        });

        let result = state.git.exec(&project.path, &["pull", "--no-edit"]).await;
        results.push(result.clone());

        let status = if result.success {
            "success"
        } else {
            "error"
        };

        let _ = window.emit("git:progress", OperationEvent {
            task_id,
            status: status.to_string(),
            message: Some(if result.success {
                format!("成功拉取 {}", project.name)
            } else {
                format!("拉取 {} 失败：{}", project.name, result.stderr)
            }),
        });
    }

    Ok(results)
}

/// 批量 Fetch
#[tauri::command]
pub async fn batch_fetch(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<Vec<GitResult>, String> {
    let projects: Vec<_> = {
        let config = state.config.read();
        config.projects
            .iter()
            .filter(|p| project_ids.contains(&p.id))
            .cloned()
            .collect()
    };

    let mut results = Vec::new();

    for project in &projects {
        let task_id = Uuid::new_v4().to_string();
        
        let _ = window.emit("git:progress", OperationEvent {
            task_id: task_id.clone(),
            status: "running".to_string(),
            message: Some(format!("正在获取 {}...", project.name)),
        });

        let result = state.git.exec(&project.path, &["fetch", "--prune", "--all"]).await;
        results.push(result.clone());

        let status = if result.success {
            "success"
        } else {
            "error"
        };

        let _ = window.emit("git:progress", OperationEvent {
            task_id,
            status: status.to_string(),
            message: Some(if result.success {
                format!("成功获取 {}", project.name)
            } else {
                format!("获取 {} 失败：{}", project.name, result.stderr)
            }),
        });
    }

    Ok(results)
}

/// 批量 Push
#[tauri::command]
pub async fn batch_push(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<Vec<GitResult>, String> {
    let projects: Vec<_> = {
        let config = state.config.read();
        config.projects
            .iter()
            .filter(|p| project_ids.contains(&p.id))
            .cloned()
            .collect()
    };

    let mut results = Vec::new();

    for project in &projects {
        let task_id = Uuid::new_v4().to_string();

        let _ = window.emit("git:progress", OperationEvent {
            task_id: task_id.clone(),
            status: "running".to_string(),
            message: Some(format!("正在推送 {}...", project.name)),
        });

        // 无上游分支时直接报错，避免 git 交互式提示卡死
        let upstream = state
            .git
            .exec(
                &project.path,
                &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
            )
            .await;
        if !upstream.success || upstream.stdout.trim().is_empty() {
            let msg = "当前分支尚未设置上游分支，无法推送".to_string();
            results.push(GitResult {
                success: false,
                stdout: String::new(),
                stderr: msg.clone(),
                duration_ms: 0,
            });
            let _ = window.emit("git:progress", OperationEvent {
                task_id,
                status: "error".to_string(),
                message: Some(format!("推送 {} 失败：{}", project.name, msg)),
            });
            continue;
        }

        let result = state.git.exec(&project.path, &["push"]).await;
        results.push(result.clone());

        let status = if result.success {
            "success"
        } else {
            "error"
        };

        let _ = window.emit("git:progress", OperationEvent {
            task_id,
            status: status.to_string(),
            message: Some(if result.success {
                format!("成功推送 {}", project.name)
            } else {
                format!("推送 {} 失败：{}", project.name, result.stderr)
            }),
        });
    }

    Ok(results)
}

/// 校验 SHA 是否为合法的十六进制提交哈希（长度 1~64）
fn is_safe_sha(sha: &str) -> bool {
    !sha.is_empty() && sha.len() <= 64 && sha.chars().all(|c| c.is_ascii_hexdigit())
}

/// 打开仓库文件夹
#[tauri::command]
pub async fn open_repo_folder(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.read();
    let project = config.projects.iter()
        .find(|p| p.id == project_id)
        .ok_or("未找到项目")?
        .clone();
    drop(config);

    open::that(&project.path)
        .map_err(|e| format!("打开文件夹失败：{}", e))?;
    
    Ok(())
}

/// 列出目录内容（单层，供前端懒加载文件树使用）
#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<FileNode>, String> {
    let entries = std::fs::read_dir(&path)
        .map_err(|e| format!("读取目录失败：{}", e))?;

    let mut nodes: Vec<FileNode> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        // 隐藏 .git 目录，避免噪音
        if name == ".git" {
            continue;
        }

        let entry_path = entry.path();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let has_children = if is_dir {
            std::fs::read_dir(&entry_path).map(|mut d| d.next().is_some()).unwrap_or(false)
        } else {
            false
        };

        nodes.push(FileNode {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
            has_children,
        });
    }

    // 目录在前，同级按名称（不区分大小写）排序
    nodes.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(nodes)
}

/// 读取文件内容（仅文本，超过 5MB 或二进制/NUL 字节会拒绝）
#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    const MAX_SIZE: u64 = 5 * 1024 * 1024;

    let metadata = std::fs::metadata(&path).map_err(|e| format!("读取文件失败：{}", e))?;
    if metadata.len() > MAX_SIZE {
        return Err("文件过大（超过 5MB），无法在编辑器中打开".to_string());
    }

    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败：{}", e))?;

    // 前若干字节含 NUL 基本可判定为二进制
    if bytes.iter().take(8000).any(|b| *b == 0) {
        return Err("二进制文件无法在编辑器中打开".to_string());
    }

    String::from_utf8(bytes).map_err(|_| "文件编码不是 UTF-8".to_string())
}

/// 写入文件内容
#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("保存文件失败：{}", e))
}

// ============ Git 单仓库操作命令（staging / diff / commit / branch）============

/// 按 ID 解析项目配置
fn resolve_project(state: &AppState, project_id: &str) -> Result<Project, String> {
    let config = state.config.read();
    config
        .projects
        .iter()
        .find(|p| p.id == project_id)
        .cloned()
        .ok_or_else(|| "未找到项目".to_string())
}

/// 暂存指定文件
#[tauri::command]
pub async fn git_stage(
    project_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project = resolve_project(&state, &project_id)?;
    if paths.is_empty() {
        return Ok(());
    }
    let mut args: Vec<String> = vec!["add".into(), "--".into()];
    for p in &paths {
        args.push(p.clone());
    }
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = state.git.exec(&project.path, &arg_refs).await;
    if !result.success {
        return Err(format!("暂存失败：{}", result.stderr));
    }
    state.cache.invalidate(&project_id);
    Ok(())
}

/// 取消暂存指定文件
#[tauri::command]
pub async fn git_unstage(
    project_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project = resolve_project(&state, &project_id)?;
    if paths.is_empty() {
        return Ok(());
    }
    let mut args: Vec<String> = vec!["reset".into(), "HEAD".into(), "--".into()];
    for p in &paths {
        args.push(p.clone());
    }
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = state.git.exec(&project.path, &arg_refs).await;
    if result.success {
        state.cache.invalidate(&project_id);
        return Ok(());
    }
    // 仓库尚无任何提交（无 HEAD）时改用 rm --cached
    let stderr = result.stderr.to_lowercase();
    if stderr.contains("ambiguous argument")
        || stderr.contains("unknown revision")
        || stderr.contains("does not have any commits")
    {
        let mut rm_args: Vec<String> = vec!["rm".into(), "--cached".into(), "-r".into(), "--".into()];
        for p in &paths {
            rm_args.push(p.clone());
        }
        let rm_refs: Vec<&str> = rm_args.iter().map(|s| s.as_str()).collect();
        let r = state.git.exec(&project.path, &rm_refs).await;
        if !r.success {
            return Err(format!("取消暂存失败：{}", r.stderr));
        }
        state.cache.invalidate(&project_id);
        return Ok(());
    }
    Err(format!("取消暂存失败：{}", result.stderr))
}

/// 丢弃改动（已跟踪文件 restore / 未跟踪文件 clean）
#[tauri::command]
pub async fn git_discard(
    project_id: String,
    entries: Vec<DiscardEntry>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project = resolve_project(&state, &project_id)?;
    if entries.is_empty() {
        return Ok(());
    }
    let mut tracked: Vec<String> = Vec::new();
    let mut untracked: Vec<String> = Vec::new();
    for e in &entries {
        if e.untracked {
            untracked.push(e.path.clone());
        } else {
            tracked.push(e.path.clone());
        }
    }
    if !tracked.is_empty() {
        let mut args: Vec<String> = vec!["restore".into(), "--worktree".into(), "--".into()];
        for p in &tracked {
            args.push(p.clone());
        }
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let r = state.git.exec(&project.path, &refs).await;
        if !r.success {
            return Err(format!("丢弃改动失败：{}", r.stderr));
        }
    }
    if !untracked.is_empty() {
        let mut args: Vec<String> = vec!["clean".into(), "-f".into(), "-d".into(), "--".into()];
        for p in &untracked {
            args.push(p.clone());
        }
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let r = state.git.exec(&project.path, &refs).await;
        if !r.success {
            return Err(format!("清理未跟踪文件失败：{}", r.stderr));
        }
    }
    state.cache.invalidate(&project_id);
    Ok(())
}

/// 提交（返回新提交 sha + 摘要）
#[tauri::command]
pub async fn git_commit(
    project_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<GitCommitResult, String> {
    let project = resolve_project(&state, &project_id)?;
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err("提交信息不能为空".to_string());
    }
    let args = ["commit", "-m", trimmed];
    let result = state.git.exec(&project.path, &args).await;
    if !result.success {
        return Err(format!("提交失败：{}", result.stderr));
    }
    let show = state
        .git
        .exec(&project.path, &["show", "-s", "--format=%H%n%s", "HEAD"])
        .await;
    let (commit_sha, summary) = if show.success {
        let mut lines = show.stdout.lines();
        let sha = lines.next().unwrap_or("").to_string();
        let sum = lines.next().unwrap_or("").to_string();
        (sha, sum)
    } else {
        (String::new(), String::new())
    };
    state.cache.invalidate(&project_id);
    Ok(GitCommitResult {
        commit_sha,
        summary,
    })
}

/// 获取 diff（原始 patch 文本）
#[tauri::command]
pub async fn git_diff(
    project_id: String,
    path: Option<String>,
    staged: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let project = resolve_project(&state, &project_id)?;
    let mut args: Vec<String> = vec!["diff".into(), "--no-ext-diff".into()];
    if staged {
        args.push("--cached".into());
    }
    if let Some(p) = path.filter(|p| !p.is_empty()) {
        args.push("--".into());
        args.push(p);
    }
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = state.git.exec(&project.path, &refs).await;
    if !result.success {
        return Err(format!("获取 diff 失败：{}", result.stderr));
    }
    Ok(result.stdout)
}

/// 获取单文件左右对比内容 diff
#[tauri::command]
pub async fn git_diff_content(
    project_id: String,
    path: String,
    staged: bool,
    original_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<GitDiffContentResult, String> {
    let project = resolve_project(&state, &project_id)?;
    let repo = project.path.clone();

    let original_content = if staged {
        let spec = original_path
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| path.clone());
        let r = state
            .git
            .exec(&repo, &["show", &format!("HEAD:{}", spec)])
            .await;
        if r.success {
            r.stdout
        } else {
            String::new()
        }
    } else {
        let r = state.git.exec(&repo, &["show", &format!(":{}", path)]).await;
        if r.success {
            r.stdout
        } else {
            String::new()
        }
    };

    let modified_content = if staged {
        let r = state.git.exec(&repo, &["show", &format!(":{}", path)]).await;
        r.stdout
    } else {
        std::fs::read_to_string(std::path::Path::new(&repo).join(&path)).unwrap_or_default()
    };

    let is_binary = original_content.contains('\0') || modified_content.contains('\0');

    let patch_args: Vec<String> = if staged {
        vec![
            "diff".into(),
            "--no-ext-diff".into(),
            "--cached".into(),
            "--".into(),
            path.clone(),
        ]
    } else {
        vec![
            "diff".into(),
            "--no-ext-diff".into(),
            "--".into(),
            path.clone(),
        ]
    };
    let patch_refs: Vec<&str> = patch_args.iter().map(|s| s.as_str()).collect();
    let patch = state.git.exec(&repo, &patch_refs).await;
    let fallback_patch = if patch.success {
        patch.stdout
    } else {
        String::new()
    };

    Ok(GitDiffContentResult {
        original_content,
        modified_content,
        is_binary,
        fallback_patch,
    })
}

/// 获取单次提交的完整 patch（含 stat）
#[tauri::command]
pub async fn git_show_commit(
    project_id: String,
    sha: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let project = resolve_project(&state, &project_id)?;
    if !is_safe_sha(&sha) {
        return Err("无效的提交标识".to_string());
    }
    let args = [
        "show",
        "--no-color",
        "--no-ext-diff",
        "--patch-with-stat",
        &sha,
        "--",
    ];
    let result = state.git.exec(&project.path, &args).await;
    if !result.success {
        return Err(format!("获取提交 diff 失败：{}", result.stderr));
    }
    Ok(result.stdout)
}

/// 解析 `diff-tree --name-status -z` 输出为逐文件改动（仅状态与路径）
fn parse_name_status(bytes: &str) -> Vec<CommitFile> {
    let tokens: Vec<&str> = bytes.split('\0').filter(|t| !t.is_empty()).collect();
    let mut files: Vec<CommitFile> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let status_tok = tokens[i];
        i += 1;
        let status_char = status_tok.chars().next().unwrap_or(' ');
        if status_char == 'R' || status_char == 'C' {
            if i + 1 < tokens.len() {
                let original = tokens[i].to_string();
                i += 1;
                let new_path = tokens[i].to_string();
                i += 1;
                files.push(CommitFile {
                    status: status_char.to_string(),
                    path: new_path,
                    original_path: Some(original),
                    added: 0,
                    removed: 0,
                    is_binary: false,
                });
            }
        } else if i < tokens.len() {
            let p = tokens[i].to_string();
            i += 1;
            files.push(CommitFile {
                status: status_char.to_string(),
                path: p,
                original_path: None,
                added: 0,
                removed: 0,
                is_binary: false,
            });
        }
    }
    files
}

/// 将 `diff-tree --numstat -z` 的增删行数 / 改名信息合并进文件列表
fn apply_numstat(files: &mut Vec<CommitFile>, bytes: &str) {
    let tokens: Vec<&str> = bytes.split('\0').filter(|t| !t.is_empty()).collect();
    let mut i = 0;
    while i < tokens.len() {
        let header = tokens[i];
        i += 1;
        let mut cols = header.splitn(3, '\t');
        let added_raw = cols.next().unwrap_or("0");
        let removed_raw = cols.next().unwrap_or("0");
        let inline_path = cols.next().unwrap_or("");
        let is_binary = added_raw == "-" && removed_raw == "-";
        let added: u32 = if is_binary {
            0
        } else {
            added_raw.parse().unwrap_or(0)
        };
        let removed: u32 = if is_binary {
            0
        } else {
            removed_raw.parse().unwrap_or(0)
        };
        let (path, original) = if inline_path.is_empty() {
            let original = if i < tokens.len() {
                tokens[i].to_string()
            } else {
                String::new()
            };
            i += 1;
            let new_path = if i < tokens.len() {
                tokens[i].to_string()
            } else {
                String::new()
            };
            i += 1;
            (new_path, Some(original))
        } else {
            (inline_path.to_string(), None)
        };
        if path.is_empty() {
            continue;
        }
        if let Some(f) = files.iter_mut().find(|f| f.path == path) {
            f.added = added;
            f.removed = removed;
            f.is_binary = is_binary;
            if f.original_path.is_none() {
                if let Some(orig) = original {
                    if !orig.is_empty() && orig != f.path {
                        f.original_path = Some(orig);
                    }
                }
            }
        }
    }
}

/// 加载单次提交的文件改动列表（分两次调用 name-status + numstat 后合并）
async fn load_commit_files(git: &GitExecutor, repo: &str, sha: &str) -> Vec<CommitFile> {
    let ns = git
        .exec(repo, &["diff-tree", "--no-commit-id", "-r", "-z", "--name-status", sha])
        .await;
    let mut files = if ns.success {
        parse_name_status(&ns.stdout)
    } else {
        Vec::new()
    };
    let nm = git
        .exec(repo, &["diff-tree", "--no-commit-id", "-r", "-z", "--numstat", sha])
        .await;
    if nm.success {
        apply_numstat(&mut files, &nm.stdout);
    }
    files
}

/// 获取单次提交改动的文件列表（含增删行数与改名）
#[tauri::command]
pub async fn git_commit_files(
    project_id: String,
    sha: String,
    state: State<'_, AppState>,
) -> Result<Vec<CommitFile>, String> {
    let project = resolve_project(&state, &project_id)?;
    if !is_safe_sha(&sha) {
        return Err("无效的提交标识".to_string());
    }
    Ok(load_commit_files(&state.git, &project.path, &sha).await)
}

/// 获取单次提交中单个文件的左右对比内容 diff
#[tauri::command]
pub async fn git_commit_file_diff(
    project_id: String,
    sha: String,
    path: String,
    original_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<GitDiffContentResult, String> {
    let project = resolve_project(&state, &project_id)?;
    if !is_safe_sha(&sha) {
        return Err("无效的提交标识".to_string());
    }
    let repo = project.path.clone();
    let original_path_resolved = original_path
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| path.clone());

    let parent = state
        .git
        .exec(&repo, &["rev-parse", &format!("{}^", sha)])
        .await;
    let parent_ok = parent.success && !parent.stdout.trim().is_empty();
    let parent_sha = parent.stdout.trim().to_string();

    let original_content = if parent_ok {
        let spec = format!("{}:{}", parent_sha, original_path_resolved);
        let r = state.git.exec(&repo, &["show", &spec]).await;
        r.stdout
    } else {
        String::new()
    };

    let modified_res = state
        .git
        .exec(&repo, &["show", &format!("{}:{}", sha, path)])
        .await;
    let modified_content = modified_res.stdout;

    let mut patch_args: Vec<String> = vec![
        "show".into(),
        "--no-color".into(),
        "--no-ext-diff".into(),
        "--format=".into(),
        "-m".into(),
        "--first-parent".into(),
        sha.clone(),
        "--".into(),
        path.clone(),
    ];
    if original_path_resolved != path {
        patch_args.push(original_path_resolved.clone());
    }
    let patch_refs: Vec<&str> = patch_args.iter().map(|s| s.as_str()).collect();
    let patch = state.git.exec(&repo, &patch_refs).await;
    let fallback_patch = if patch.success {
        patch.stdout
    } else {
        String::new()
    };

    let is_binary = original_content.contains('\0') || modified_content.contains('\0');
    Ok(GitDiffContentResult {
        original_content,
        modified_content,
        is_binary,
        fallback_patch,
    })
}

/// 切换分支
#[tauri::command]
pub async fn git_checkout_branch(
    project_id: String,
    branch: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project = resolve_project(&state, &project_id)?;
    if branch.is_empty() || branch.starts_with('-') {
        return Err("非法的分支名".to_string());
    }
    let args = ["checkout", &branch];
    let result = state.git.exec(&project.path, &args).await;
    if !result.success {
        return Err(format!("切换分支失败：{}", result.stderr));
    }
    state.cache.invalidate(&project_id);
    Ok(())
}

/// 获取远端地址（默认 origin）
#[tauri::command]
pub async fn git_remote_url(
    project_id: String,
    name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let project = resolve_project(&state, &project_id)?;
    let remote = name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "origin".to_string());
    if !remote
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Ok(None);
    }
    let args = ["config", "--get", &format!("remote.{}.url", remote)];
    let result = state.git.exec(&project.path, &args).await;
    if result.success && !result.stdout.trim().is_empty() {
        Ok(Some(result.stdout.trim().to_string()))
    } else {
        Ok(None)
    }
}

/// 保存配置到文件
fn save_config(config: &AppConfig, cache: &StatusCache) -> Result<(), String> {
    let config_path = cache.get_config_path();

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败：{}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败：{}", e))?;
    
    std::fs::write(&config_path, json)
        .map_err(|e| format!("写入配置失败：{}", e))?;
    
    Ok(())
}

/// 打开 / 关闭 WebView 开发者工具（仅 debug 构建可用，用于前端调试）
#[tauri::command]
pub fn toggle_devtools(app: tauri::AppHandle) {
    #[cfg(debug_assertions)]
    {
        if let Some(window) = app.get_webview_window("main") {
            if window.is_devtools_open() {
                window.close_devtools();
            } else {
                window.open_devtools();
            }
        }
    }
}
