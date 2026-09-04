---
name: gitdash-migration
type: project
scope: team
description: GitDash 项目采用 Tauri 2 + Vue 3 + TypeScript + Vite 技术栈。移植工作流要求保留新项目的 `package.json`, `vite.config.ts`, `tsconfig.json` 等基础配置，仅将老项目的源码（前端组件、Pinia 状态、Rust...
created: "2026-09-03T03:30:07.543Z"
updated: "2026-09-03T03:31:00.288Z"
---
GitDash 项目采用 Tauri 2 + Vue 3 + TypeScript + Vite 技术栈。移植工作流要求保留新项目的 `package.json`, `vite.config.ts`, `tsconfig.json` 等基础配置，仅将老项目的源码（前端组件、Pinia 状态、Rust 后端、Tauri 命令）升级合并。

**Vite 配置规范**：必须使用 `import.meta.dirname` 替代 `__dirname`，以兼容 Vite v8+ 的 `configLoader: 'native'`。

**Tauri 2 权限配置**：在 `capabilities/default.json` 中显式声明所需权限，避免使用已废弃的权限名称（如 `opener:default`）。核心权限包括 `core:app:default`, `core:tray:default`, `core:window:default`；插件权限包括 `dialog:default`, `fs:default`, `shell:default`；文件系统权限需明确指定路径范围（如 `$APPDATA/`, `$HOME/`）。

**Rust 后端实现规范**：
1. 文件监听器中，`app_handle` 必须使用 `Arc<AppHandle>` 包装，以满足 `notify` 库事件处理器的 `Send + Sync` trait 要求。
2. 类型约束需使用 `W: Runtime` 和 `R: Manager<W> + Emitter<W>`，避免直接传递 `&R` 导致线程安全问题。
3. 超时控制需使用 `tokio::time::timeout` 替代 `tokio::process::Command` 不存在的 `.timeout()` 方法。
4. `main.rs` 中导入模块时，直接使用 `use commands::*; use git::GitExecutor;` 等，避免使用 `gitdash::` 前缀（库名为 `gitdash_lib`）。
5. 系统托盘 API 使用 `show_menu_on_left_click(true)` 替代已废弃的 `menu_on_left_click(true)`。