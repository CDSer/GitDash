// Tauri Commands 模块
// 定义所有暴露给前端的 Rust 命令

use crate::models::{AppConfig, Branch, Commit, CommitDetail, CommitFile, FileNode, GitResult, Group, OperationEvent, Project, ProjectStatus};
use crate::scanner::ProjectScanner;
use crate::store::{AppState, StatusCache};
use crate::watcher::WatcherManager;
use tauri::{Emitter, State, Window};
use uuid::Uuid;

/// 获取完整配置
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read();
    Ok(config.clone())
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
        is_favorite: false,
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

    let status = state.git.status(&project.path).await;
    
    if status.is_clean {
        state.cache.set(project_id.clone(), status.clone());
    }

    Ok(status)
}

/// 获取仓库分支列表
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

    let current_result = state.git.exec(&project.path, &["rev-parse", "--abbrev-ref", "HEAD"]).await;
    let current_branch = if current_result.success {
        current_result.stdout.trim().to_string()
    } else {
        String::new()
    };

    let result = state.git.exec(
        &project.path,
        &[
            "for-each-ref",
            "--format=%(refname)\t%(upstream:short)",
            "refs/heads",
            "refs/remotes",
        ],
    ).await;

    if !result.success {
        return Err(format!("获取分支失败：{}", result.stderr));
    }

    let mut branches = Vec::new();

    for line in result.stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let refname = parts[0];
        let upstream = if parts[1].is_empty() { None } else { Some(parts[1].to_string()) };

        let (full_name, display_name, is_local, is_remote) = if let Some(name) = refname.strip_prefix("refs/heads/") {
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
        });
    }

    Ok(branches)
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

    let files_result = state.git.exec(
        &project.path,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "-r",
            &commit_id,
        ],
    ).await;

    let mut files = Vec::new();
    if files_result.success {
        for line in files_result.stdout.lines() {
            if line.is_empty() {
                continue;
            }
            let mut split = line.splitn(2, '\t');
            let status = split.next().unwrap_or("").to_string();
            let path = split.next().unwrap_or("").to_string();
            if !path.is_empty() {
                files.push(CommitFile { status, path });
            }
        }
    }

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
    window: Window,
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

        let result = state.git.exec(&project.path, &["pull"]).await;
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
    window: Window,
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

        let result = state.git.exec(&project.path, &["fetch", "--all"]).await;
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
    window: Window,
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
