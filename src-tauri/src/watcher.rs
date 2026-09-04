// 文件监听器模块
// 监听 Git 仓库变化并通知前端

use crate::models::Project;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// 文件监听器管理器
pub struct WatcherManager {
    watchers: Mutex<HashMap<String, (RecommendedWatcher, Vec<PathBuf>)>>,
}

impl WatcherManager {
    /// 创建新的监听器管理器
    pub fn new() -> Self {
        Self {
            watchers: Mutex::new(HashMap::new()),
        }
    }

    /// 添加项目监听
    /// 
    /// # Arguments
    /// * `project` - 项目信息
    /// * `app_handle` - Tauri 应用句柄
    pub async fn add_project(&self, project: &Project, app_handle: &AppHandle) -> Result<(), String> {
        let mut watchers = self.watchers.lock().await;
        
        if watchers.contains_key(&project.id) {
            return Ok(());
        }

        let git_dir = PathBuf::from(&project.path).join(".git");
        if !git_dir.exists() {
            return Err(format!("不是 Git 仓库：{}", project.path));
        }

        let paths_to_watch = vec![
            git_dir.join("HEAD"),
            git_dir.join("index"),
            git_dir.join("refs").join("heads"),
            git_dir.join("refs").join("remotes"),
        ];

        // 使用 Arc<AppHandle> 来确保 Send + Sync
        let app_handle = Arc::new(app_handle.clone());
        let project_id_clone = project.id.clone();
        let event_handler = {
            let app_handle = app_handle.clone();
            move |event: Result<Event, notify::Error>| {
                if let Ok(_) = event {
                    let _ = app_handle.emit("repo:changed", project_id_clone.clone());
                }
            }
        };

        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())
            .map_err(|e| format!("创建监听器失败：{}", e))?;

        for path in &paths_to_watch {
            if path.exists() {
                watcher.watch(path, RecursiveMode::NonRecursive)
                    .map_err(|e| format!("监听路径失败：{}", e))?;
            }
        }

        watchers.insert(project.id.clone(), (watcher, paths_to_watch));
        Ok(())
    }

    /// 移除项目监听
    pub async fn remove_project(&self, project_id: &str) {
        let mut watchers = self.watchers.lock().await;
        watchers.remove(project_id);
    }

    /// 更新项目监听（当路径变化时）
    pub async fn update_project(&self, old_project: &Project, new_project: &Project, app_handle: &AppHandle) -> Result<(), String> {
        if old_project.path != new_project.path {
            self.remove_project(&old_project.id).await;
            self.add_project(new_project, app_handle).await?;
        }
        Ok(())
    }
}
