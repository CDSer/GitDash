// Tauri Commands 模块
// 定义所有暴露给前端的 Rust 命令

use crate::models::{
    AppConfig, BatchImportResult, Branch, Commit, CommitDetail, CommitFile, ConflictFileContent,
    ConflictSide, DiscardEntry, FileNode, GitCommitResult, GitDiffContentResult, Group,
    InProgressOp, MergeResult, OperationEvent, Project, ProjectGitResult, ProjectStatus,
    ScanOptions, ScannedRepo, Settings,
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
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut config = state.config.write();
        config.settings = settings.clone();
        save_config(&config, &state.cache)?;
    }
    apply_runtime_settings(&app_handle, &state, &settings);
    Ok(())
}

/// 将设置应用到运行时（Git 路径、并发数、全局快捷键）
pub fn apply_runtime_settings(
    app_handle: &tauri::AppHandle,
    state: &AppState,
    settings: &Settings,
) {
    state.git.set_git_path(settings.git_path.clone());
    state.git
        .set_max_concurrent(settings.max_concurrent_git.max(1));
    apply_global_shortcut(app_handle, &settings.global_shortcut);
}

/// 注册/替换全局快捷键（显示主窗口）
fn apply_global_shortcut(app_handle: &tauri::AppHandle, combo: &str) -> bool {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let _ = app_handle.global_shortcut().unregister_all();

    let combo = combo.trim();
    if combo.is_empty() {
        return true;
    }

    let normalized = combo
        .replace("CmdOrControl+", "CommandOrControl+")
        .replace("cmdorcontrol+", "CommandOrControl+");

    match app_handle.global_shortcut().register(normalized.as_str()) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("无法注册全局快捷键「{}」：{}", combo, e);
            false
        }
    }
}

/// 添加新项目
#[tauri::command]
pub async fn add_project(
    path: String,
    group_id: Option<String>,
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
        group_id,
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

/// 扫描目录中的 Git 仓库候选
/// 用于批量导入前的预览，返回所有发现的仓库（不包含已存在项目）
#[tauri::command]
pub async fn scan_projects(
    base_path: String,
    options: Option<ScanOptions>,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<ScannedRepo>, String> {
    let options = options.unwrap_or_default();
    let base = std::path::PathBuf::from(&base_path);

    if !base.exists() {
        return Err("目录不存在".to_string());
    }
    if !base.is_dir() {
        return Err("路径不是目录".to_string());
    }

    let blacklist = state.config.read().settings.scan_blacklist.clone();

    let scanned_repos = tokio::task::spawn_blocking({
        let base_path = base_path.clone();
        let options = options.clone();
        let blacklist = blacklist.clone();
        let app_handle = app_handle.clone();
        move || ProjectScanner::scan_directory(&base_path, &options, &blacklist, &app_handle)
    })
    .await
    .map_err(|e| format!("扫描任务异常：{}", e))?;

    // 发送最终进度事件
    let _ = app_handle.emit(
        "import:progress",
        crate::models::ImportProgressEvent {
            scanned: scanned_repos.len() as u64,
            found: scanned_repos.len() as u64,
            current: String::new(),
        },
    );

    Ok(scanned_repos)
}

/// 批量导入 Git 项目
/// 根据前端勾选的仓库路径，创建项目并注册监听
#[tauri::command]
pub async fn batch_import_projects(
    paths: Vec<String>,
    group_id: Option<String>,
    state: State<'_, AppState>,
    watcher: State<'_, WatcherManager>,
    app_handle: tauri::AppHandle,
) -> Result<BatchImportResult, String> {
    if paths.is_empty() {
        return Ok(BatchImportResult {
            added: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
        });
    }

    let existing_paths = {
        let config = state.config.read();
        ProjectScanner::existing_paths(&config.projects)
    };

    let mut added = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();
    let mut new_projects = Vec::new();

    for path in paths {
        let normalized = ProjectScanner::normalize_for_dedup(&path);

        if existing_paths.contains(&normalized) {
            skipped.push(path);
            continue;
        }

        if !ProjectScanner::is_valid_git_repo(&path) {
            failed.push(path);
            continue;
        }

        match ProjectScanner::create_project(std::path::Path::new(&path), group_id.clone()) {
            Some(project) => {
                new_projects.push(project.clone());
                added.push(project);
            }
            None => {
                failed.push(path);
            }
        }
    }

    // 写入配置
    if !new_projects.is_empty() {
        let mut config = state.config.write();
        config.projects.extend(new_projects);
        save_config(&config, &state.cache)?;
    }

    // 注册监听器
    for project in &added {
        if let Err(e) = watcher.add_project(project, &app_handle).await {
            eprintln!("注册监听器失败 {}: {}", project.path, e);
        }
    }

    Ok(BatchImportResult {
        added,
        skipped,
        failed,
    })
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

/// 批量删除项目
#[tauri::command]
pub async fn remove_projects(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    watcher: State<'_, WatcherManager>,
) -> Result<(), String> {
    let id_set: std::collections::HashSet<String> = project_ids.iter().cloned().collect();

    {
        let mut config = state.config.write();
        config.projects.retain(|p| !id_set.contains(&p.id));
        save_config(&config, &state.cache)?;
    }

    for project_id in &project_ids {
        watcher.remove_project(project_id).await;
        state.cache.invalidate(project_id);
    }

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

/// 解析一批项目（保持 config 顺序，但结果均携带 project_id）
fn resolve_projects(state: &AppState, project_ids: &[String]) -> Vec<Project> {
    let id_set: std::collections::HashSet<&str> = project_ids.iter().map(|s| s.as_str()).collect();
    let config = state.config.read();
    config
        .projects
        .iter()
        .filter(|p| id_set.contains(p.id.as_str()))
        .cloned()
        .collect()
}

/// 串行等待 JoinSet 并汇总为 ProjectGitResult
async fn collect_git_results(
    mut set: tokio::task::JoinSet<ProjectGitResult>,
) -> Vec<ProjectGitResult> {
    let mut results = Vec::with_capacity(set.len());
    while let Some(joined) = set.join_next().await {
        if let Ok(item) = joined {
            results.push(item);
        }
    }
    results
}

/// 批量 Pull（并行，受 GitExecutor 并发上限约束）
#[tauri::command]
pub async fn batch_pull(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<Vec<ProjectGitResult>, String> {
    let projects = resolve_projects(&state, &project_ids);
    let git = state.git.clone();
    let cache = state.cache.clone();
    let mut set = tokio::task::JoinSet::new();

    for project in projects {
        let git = git.clone();
        let cache = cache.clone();
        let window = window.clone();
        set.spawn(async move {
            let task_id = Uuid::new_v4().to_string();
            let _ = window.emit(
                "git:progress",
                OperationEvent {
                    task_id: task_id.clone(),
                    project_id: project.id.clone(),
                    status: "running".to_string(),
                    message: Some(format!("正在拉取 {}...", project.name)),
                },
            );

            let result = git.exec(&project.path, &["pull", "--no-edit"]).await;
            cache.invalidate(&project.id);

            let status = if result.success {
                "success"
            } else {
                "error"
            };
            let _ = window.emit(
                "git:progress",
                OperationEvent {
                    task_id,
                    project_id: project.id.clone(),
                    status: status.to_string(),
                    message: Some(if result.success {
                        format!("成功拉取 {}", project.name)
                    } else {
                        format!("拉取 {} 失败：{}", project.name, result.stderr)
                    }),
                },
            );

            ProjectGitResult {
                project_id: project.id,
                success: result.success,
                stdout: result.stdout,
                stderr: result.stderr,
                duration_ms: result.duration_ms,
            }
        });
    }

    Ok(collect_git_results(set).await)
}

/// 批量 Fetch（并行）
#[tauri::command]
pub async fn batch_fetch(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<Vec<ProjectGitResult>, String> {
    let projects = resolve_projects(&state, &project_ids);
    let git = state.git.clone();
    let cache = state.cache.clone();
    let mut set = tokio::task::JoinSet::new();

    for project in projects {
        let git = git.clone();
        let cache = cache.clone();
        let window = window.clone();
        set.spawn(async move {
            let task_id = Uuid::new_v4().to_string();
            let _ = window.emit(
                "git:progress",
                OperationEvent {
                    task_id: task_id.clone(),
                    project_id: project.id.clone(),
                    status: "running".to_string(),
                    message: Some(format!("正在获取 {}...", project.name)),
                },
            );

            let result = git.exec(&project.path, &["fetch", "--prune", "--all"]).await;
            cache.invalidate(&project.id);

            let status = if result.success {
                "success"
            } else {
                "error"
            };
            let _ = window.emit(
                "git:progress",
                OperationEvent {
                    task_id,
                    project_id: project.id.clone(),
                    status: status.to_string(),
                    message: Some(if result.success {
                        format!("成功获取 {}", project.name)
                    } else {
                        format!("获取 {} 失败：{}", project.name, result.stderr)
                    }),
                },
            );

            ProjectGitResult {
                project_id: project.id,
                success: result.success,
                stdout: result.stdout,
                stderr: result.stderr,
                duration_ms: result.duration_ms,
            }
        });
    }

    Ok(collect_git_results(set).await)
}

/// 批量 Push（并行）
#[tauri::command]
pub async fn batch_push(
    project_ids: Vec<String>,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<Vec<ProjectGitResult>, String> {
    let projects = resolve_projects(&state, &project_ids);
    let git = state.git.clone();
    let cache = state.cache.clone();
    let mut set = tokio::task::JoinSet::new();

    for project in projects {
        let git = git.clone();
        let cache = cache.clone();
        let window = window.clone();
        set.spawn(async move {
            let task_id = Uuid::new_v4().to_string();
            let _ = window.emit(
                "git:progress",
                OperationEvent {
                    task_id: task_id.clone(),
                    project_id: project.id.clone(),
                    status: "running".to_string(),
                    message: Some(format!("正在推送 {}...", project.name)),
                },
            );

            // 无上游分支时直接报错，避免 git 交互式提示卡死
            let upstream = git
                .exec(
                    &project.path,
                    &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
                )
                .await;
            if !upstream.success || upstream.stdout.trim().is_empty() {
                let msg = "当前分支尚未设置上游分支，无法推送".to_string();
                let _ = window.emit(
                    "git:progress",
                    OperationEvent {
                        task_id,
                        project_id: project.id.clone(),
                        status: "error".to_string(),
                        message: Some(format!("推送 {} 失败：{}", project.name, msg)),
                    },
                );
                return ProjectGitResult {
                    project_id: project.id,
                    success: false,
                    stdout: String::new(),
                    stderr: msg,
                    duration_ms: 0,
                };
            }

            let result = git.exec(&project.path, &["push"]).await;
            cache.invalidate(&project.id);

            let status = if result.success {
                "success"
            } else {
                "error"
            };
            let _ = window.emit(
                "git:progress",
                OperationEvent {
                    task_id,
                    project_id: project.id.clone(),
                    status: status.to_string(),
                    message: Some(if result.success {
                        format!("成功推送 {}", project.name)
                    } else {
                        format!("推送 {} 失败：{}", project.name, result.stderr)
                    }),
                },
            );

            ProjectGitResult {
                project_id: project.id,
                success: result.success,
                stdout: result.stdout,
                stderr: result.stderr,
                duration_ms: result.duration_ms,
            }
        });
    }

    Ok(collect_git_results(set).await)
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

/// 校验路径是否位于任一受管项目根目录内（拒绝 `..` 逃逸与任意系统路径）
fn path_within_projects(state: &AppState, path: &str) -> Result<std::path::PathBuf, String> {
    let raw = std::path::PathBuf::from(path);
    // 禁止显式 .. 片段（规范化前先拦一层）
    if path.split(['/', '\\']).any(|seg| seg == "..") {
        return Err("路径不允许包含 ..".to_string());
    }

    let projects: Vec<String> = {
        let config = state.config.read();
        config.projects.iter().map(|p| p.path.clone()).collect()
    };

    let canon = raw
        .canonicalize()
        .map_err(|e| format!("路径无效：{}", e))?;

    for root in &projects {
        if let Ok(root_canon) = std::path::Path::new(root).canonicalize() {
            if canon.starts_with(&root_canon) {
                return Ok(canon);
            }
        }
    }

    Err("路径不在任何已管理项目内".to_string())
}

/// 列出目录内容（单层，供前端懒加载文件树使用）
/// 仅允许访问已添加项目的根目录内路径
#[tauri::command]
pub async fn list_directory(path: String, state: State<'_, AppState>) -> Result<Vec<FileNode>, String> {
    let path = path_within_projects(&state, &path)?;
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
/// 仅允许读取已管理项目内的文件
#[tauri::command]
pub async fn read_file(path: String, state: State<'_, AppState>) -> Result<String, String> {
    const MAX_SIZE: u64 = 5 * 1024 * 1024;

    let path = path_within_projects(&state, &path)?;
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
/// 仅允许写入已管理项目内的文件
#[tauri::command]
pub async fn write_file(path: String, content: String, state: State<'_, AppState>) -> Result<(), String> {
    let path = path_within_projects(&state, &path)?;
    if path.is_dir() {
        return Err("目标是目录，无法写入".to_string());
    }
    // 拒绝写入 .git 内部
    if path.components().any(|c| c.as_os_str() == ".git") {
        return Err("不允许写入 .git 目录内的文件".to_string());
    }
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
/// 存在未解决冲突时拒绝；进行中的 merge 在全部 resolved 后可用 message 或默认信息完成合并提交
#[tauri::command]
pub async fn git_commit(
    project_id: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<GitCommitResult, String> {
    let project = resolve_project(&state, &project_id)?;

    // 检查未合并路径
    let unmerged = state
        .git
        .exec(&project.path, &["ls-files", "-u"])
        .await;
    if unmerged.success && !unmerged.stdout.trim().is_empty() {
        return Err("存在未解决的合并冲突，请先解决后再提交".to_string());
    }

    let in_progress = crate::git::GitExecutor::detect_in_progress_op(&project.path);
    let trimmed = message.trim();

    // merge 进行中且未提供信息时，用 git 自带的 MERGE_MSG 完成提交
    let is_merge_finish = matches!(in_progress.as_ref().map(|p| p.kind.as_str()), Some("merge"));
    if trimmed.is_empty() {
        if !is_merge_finish {
            return Err("提交信息不能为空".to_string());
        }
    }

    let result = if is_merge_finish && trimmed.is_empty() {
        state
            .git
            .exec(&project.path, &["commit", "--no-edit"])
            .await
    } else {
        let args = ["commit", "-m", trimmed];
        state.git.exec(&project.path, &args).await
    };

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

// ============ 冲突 / 合并解决 ============

/// 查询进行中的 merge / rebase / cherry-pick
#[tauri::command]
pub async fn git_in_progress(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Option<InProgressOp>, String> {
    let project = resolve_project(&state, &project_id)?;
    Ok(crate::git::GitExecutor::detect_in_progress_op(
        &project.path,
    ))
}

/// 发起合并（目标分支名）
#[tauri::command]
pub async fn git_merge(
    project_id: String,
    branch: String,
    state: State<'_, AppState>,
) -> Result<MergeResult, String> {
    let project = resolve_project(&state, &project_id)?;
    if branch.is_empty() || branch.starts_with('-') {
        return Err("非法的分支名".to_string());
    }

    let in_progress = crate::git::GitExecutor::detect_in_progress_op(&project.path);
    if in_progress.is_some() {
        return Err("已有进行中的合并/变基操作，请先完成或中止".to_string());
    }

    // 工作区有未暂存改动时拒绝 merge，避免 git 交互/污染
    let status = state.git.status(&project.path).await;
    if status.modified > 0 || status.untracked > 0 || status.conflict_count > 0 {
        return Err("工作区有未提交的改动，请先提交或暂存后再合并".to_string());
    }

    let result = state
        .git
        .exec(&project.path, &["merge", "--no-edit", &branch])
        .await;

    let has_conflicts = {
        let ls = state.git.exec(&project.path, &["ls-files", "-u"]).await;
        ls.success && !ls.stdout.trim().is_empty()
    };

    state.cache.invalidate(&project_id);

    if !result.success && !has_conflicts {
        return Err(format!("合并失败：{}", result.stderr));
    }

    Ok(MergeResult {
        success: result.success || has_conflicts,
        has_conflicts,
        message: if has_conflicts {
            "合并存在冲突，请在源码控制中解决".to_string()
        } else {
            result.stdout.trim().to_string()
        },
    })
}

/// 中止进行中的 merge / rebase / cherry-pick / revert
#[tauri::command]
pub async fn git_abort_operation(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project = resolve_project(&state, &project_id)?;
    let in_progress = crate::git::GitExecutor::detect_in_progress_op(&project.path)
        .ok_or_else(|| "当前没有进行中的合并/变基操作".to_string())?;

    let args: Vec<&str> = match in_progress.kind.as_str() {
        "merge" => vec!["merge", "--abort"],
        "rebase" => vec!["rebase", "--abort"],
        "cherry-pick" => vec!["cherry-pick", "--abort"],
        "revert" => vec!["revert", "--abort"],
        other => return Err(format!("不支持中止的操作：{}", other)),
    };

    let result = state.git.exec(&project.path, &args).await;
    state.cache.invalidate(&project_id);
    if !result.success {
        return Err(format!("中止失败：{}", result.stderr));
    }
    Ok(())
}

/// 继续进行中的 merge / rebase / cherry-pick / revert
/// merge：在冲突已解决后提交；其余走对应 continue
#[tauri::command]
pub async fn git_merge_continue(
    project_id: String,
    message: Option<String>,
    state: State<'_, AppState>,
) -> Result<GitCommitResult, String> {
    let project = resolve_project(&state, &project_id)?;
    let in_progress = crate::git::GitExecutor::detect_in_progress_op(&project.path)
        .ok_or_else(|| "当前没有进行中的合并/变基操作".to_string())?;

    let unmerged = state.git.exec(&project.path, &["ls-files", "-u"]).await;
    if unmerged.success && !unmerged.stdout.trim().is_empty() {
        return Err("仍有未解决的冲突文件".to_string());
    }

    let result = match in_progress.kind.as_str() {
        "merge" => match message.map(|m| m.trim().to_string()) {
            Some(msg) if !msg.is_empty() => {
                let args = ["commit", "-m", msg.as_str()];
                state.git.exec(&project.path, &args).await
            }
            _ => state.git.exec(&project.path, &["commit", "--no-edit"]).await,
        },
        "rebase" => {
            // 停在冲突点时通常应使用 --continue；此处仅在无冲突残留时推进
            state.git.exec(&project.path, &["rebase", "--continue"]).await
        }
        "cherry-pick" => state
            .git
            .exec(&project.path, &["cherry-pick", "--continue"])
            .await,
        "revert" => state.git.exec(&project.path, &["revert", "--continue"]).await,
        other => {
            return Err(format!("不支持继续该操作：{}", other));
        }
    };

    if !result.success {
        return Err(format!("完成操作失败：{}", result.stderr));
    }

    let show = state
        .git
        .exec(&project.path, &["show", "-s", "--format=%H%n%s", "HEAD"])
        .await;
    let (commit_sha, summary) = if show.success {
        let mut lines = show.stdout.lines();
        (
            lines.next().unwrap_or("").to_string(),
            lines.next().unwrap_or("").to_string(),
        )
    } else {
        (String::new(), String::new())
    };

    state.cache.invalidate(&project_id);
    Ok(GitCommitResult {
        commit_sha,
        summary,
    })
}

/// 采纳一侧解决冲突（ours=当前分支，theirs=被合入分支），并自动 `git add`
#[tauri::command]
pub async fn git_resolve_conflict(
    project_id: String,
    paths: Vec<String>,
    side: ConflictSide,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project = resolve_project(&state, &project_id)?;
    if paths.is_empty() {
        return Ok(());
    }

    let side_flag = match side {
        ConflictSide::Ours => "--ours",
        ConflictSide::Theirs => "--theirs",
    };

    let mut checkout_args: Vec<String> = vec!["checkout".into(), side_flag.into(), "--".into()];
    for p in &paths {
        checkout_args.push(p.clone());
    }
    let refs: Vec<&str> = checkout_args.iter().map(|s| s.as_str()).collect();
    let r = state.git.exec(&project.path, &refs).await;

    // checkout 失败时仍尝试 add：例如 delete/modify 冲突一侧无文件内容
    let mut add_args: Vec<String> = vec!["add".into(), "--".into()];
    for p in &paths {
        add_args.push(p.clone());
    }
    let add_refs: Vec<&str> = add_args.iter().map(|s| s.as_str()).collect();
    let a = state.git.exec(&project.path, &add_refs).await;
    if !a.success {
        let detail = if !r.success {
            format!("{} | {}", r.stderr.trim(), a.stderr.trim())
        } else {
            a.stderr.trim().to_string()
        };
        return Err(format!(
            "采纳「{}」失败：{}",
            match side {
                ConflictSide::Ours => "我方",
                ConflictSide::Theirs => "对方",
            },
            detail
        ));
    }

    state.cache.invalidate(&project_id);
    Ok(())
}

/// 手动编辑后标记冲突已解决（git add）
#[tauri::command]
pub async fn git_mark_conflict_resolved(
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
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let r = state.git.exec(&project.path, &refs).await;
    if !r.success {
        return Err(format!("标记已解决失败：{}", r.stderr));
    }
    state.cache.invalidate(&project_id);
    Ok(())
}

/// 读取冲突文件的三路内容（base / ours / theirs）
#[tauri::command]
pub async fn git_conflict_file_content(
    project_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<ConflictFileContent, String> {
    let project = resolve_project(&state, &project_id)?;

    async fn show_stage(
        git: &crate::git::GitExecutor,
        repo: &str,
        stage: u8,
        path: &str,
    ) -> (String, bool) {
        let spec = format!(":{}:{}", stage, path);
        let r = git.exec(repo, &["show", &spec]).await;
        if r.success {
            (r.stdout, true)
        } else {
            (String::new(), false)
        }
    }

    let (base, exists_base) = show_stage(&state.git, &project.path, 1, &path).await;
    let (ours, exists_ours) = show_stage(&state.git, &project.path, 2, &path).await;
    let (theirs, exists_theirs) = show_stage(&state.git, &project.path, 3, &path).await;

    let is_binary =
        base.contains('\0') || ours.contains('\0') || theirs.contains('\0');

    Ok(ConflictFileContent {
        path,
        base,
        ours,
        theirs,
        exists_base,
        exists_ours,
        exists_theirs,
        is_binary,
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
