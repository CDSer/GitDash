// Tauri Commands 模块
// 定义所有暴露给前端的 Rust 命令

use crate::models::{AppConfig, GitResult, Group, OperationEvent, Project, ProjectStatus};
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
