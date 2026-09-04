// 项目扫描器模块
// 用于扫描目录中的 Git 仓库

use crate::models::Project;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// 项目扫描器
pub struct ProjectScanner;

impl ProjectScanner {
    /// 扫描目录中的 Git 仓库
    /// 
    /// # Arguments
    /// * `base_path` - 基础目录路径
    /// 
    /// # Returns
    /// 发现的 Git 仓库列表
    pub fn scan_directory(base_path: &str) -> Vec<Project> {
        let mut projects = Vec::new();
        
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let git_dir = path.join(".git");
                    if git_dir.exists() {
                        if let Some(project) = Self::create_project(&path) {
                            projects.push(project);
                        }
                    }
                    
                    // 递归扫描子目录（限制深度）
                    projects.extend(Self::scan_subdirectories(&path, 2));
                }
            }
        }
        
        projects
    }

    /// 递归扫描子目录
    fn scan_subdirectories(base_path: &PathBuf, depth: u32) -> Vec<Project> {
        if depth == 0 {
            return Vec::new();
        }

        let mut projects = Vec::new();
        
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let git_dir = path.join(".git");
                    if git_dir.exists() {
                        if let Some(project) = Self::create_project(&path) {
                            projects.push(project);
                        }
                    } else {
                        projects.extend(Self::scan_subdirectories(&path, depth - 1));
                    }
                }
            }
        }
        
        projects
    }

    /// 创建项目对象
    fn create_project(path: &PathBuf) -> Option<Project> {
        let name = path.file_name()?.to_str()?.to_string();
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().timestamp();

        Some(Project {
            id,
            name,
            path: path.to_string_lossy().to_string(),
            group_id: None,
            tags: Vec::new(),
            is_favorite: false,
            created_at,
        })
    }

    /// 验证是否为有效的 Git 仓库
    /// 
    /// # Arguments
    /// * `path` - 路径
    /// 
    /// # Returns
    /// 是否为有效的 Git 仓库
    pub fn is_valid_git_repo(path: &str) -> bool {
        let git_dir = PathBuf::from(path).join(".git");
        git_dir.exists()
    }
}
