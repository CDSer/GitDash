// 状态存储模块
// 管理应用配置、状态缓存

use crate::git::GitExecutor;
use crate::models::{default_scan_blacklist, AppConfig, Settings};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 状态缓存条目
#[derive(Clone)]
struct CacheEntry {
    status: crate::models::ProjectStatus,
    expires_at: Instant,
}

/// 状态缓存（TTL 缓存）
pub struct StatusCache {
    cache: Mutex<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

impl StatusCache {
    /// 创建新的状态缓存
    /// 
    /// # Arguments
    /// * `ttl_seconds` - 缓存有效期（秒）
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// 获取缓存的状态
    pub fn get(&self, project_id: &str) -> Option<crate::models::ProjectStatus> {
        let cache = self.cache.lock();
        if let Some(entry) = cache.get(project_id) {
            if entry.expires_at > Instant::now() {
                return Some(entry.status.clone());
            }
        }
        None
    }

    /// 设置缓存
    pub fn set(&self, project_id: String, status: crate::models::ProjectStatus) {
        let mut cache = self.cache.lock();
        cache.insert(
            project_id,
            CacheEntry {
                status,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    /// 使缓存失效
    pub fn invalidate(&self, project_id: &str) {
        let mut cache = self.cache.lock();
        cache.remove(project_id);
    }

    /// 获取配置文件路径
    pub fn get_config_path(&self) -> PathBuf {
        let app_data_dir = Self::get_app_data_dir();
        app_data_dir.join("config.json")
    }

    /// 获取应用数据目录
    fn get_app_data_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(app_data).join("GitDash")
        }
        
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join("Library/Application Support/GitDash")
        }
        
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config/GitDash")
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            PathBuf::from(".").join(".gitdash")
        }
    }
}

/// 应用状态（Tauri State）
pub struct AppState {
    /// 应用配置（项目、分组、设置）
    pub config: Arc<RwLock<AppConfig>>,
    /// Git 执行器
    pub git: Arc<GitExecutor>,
    /// 状态缓存
    pub cache: Arc<StatusCache>,
    /// 操作队列
    pub operation_queue: Mutex<Vec<String>>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(git: Arc<GitExecutor>) -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            git,
            cache: Arc::new(StatusCache::new(5)),
            operation_queue: Mutex::new(Vec::new()),
        }
    }

    /// 加载配置文件
    pub fn load_config(&self) -> Result<(), String> {
        let config_path = StatusCache::get_config_path(&self.cache);
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("读取配置失败：{}", e))?;
            let config: AppConfig = serde_json::from_str(&content)
                .map_err(|e| format!("解析配置失败：{}", e))?;
            *self.config.write() = config;
        } else {
            let default_config = self.create_default_config();
            self.save_config_internal(&default_config, &config_path)?;
            *self.config.write() = default_config;
        }
        
        Ok(())
    }

    /// 保存配置
    pub fn save_config(&self) -> Result<(), String> {
        let config_path = StatusCache::get_config_path(&self.cache);
        let config = self.config.read().clone();
        self.save_config_internal(&config, &config_path)
    }

    /// 内部保存配置方法
    fn save_config_internal(&self, config: &AppConfig, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败：{}", e))?;
        }
        
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化配置失败：{}", e))?;
        fs::write(path, content)
            .map_err(|e| format!("写入配置失败：{}", e))?;
        
        Ok(())
    }

    /// 创建默认配置
    /// 
    /// 分组默认为空：前端会展示「全部 / 收藏 / 未分组」三个系统分组，
    /// 这里只存放用户自建的分组，避免出现重复项
    pub fn create_default_config(&self) -> AppConfig {
        AppConfig {
            version: "1.0.0".to_string(),
            projects: vec![],
            groups: vec![],
            settings: Settings {
                git_path: None,
                auto_fetch_interval: 30,
                max_concurrent_git: 3,
                theme: "system".to_string(),
                global_shortcut: "CommandOrControl+Shift+G".to_string(),
                scan_blacklist: default_scan_blacklist(),
            },
        }
    }
}
