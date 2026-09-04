# GitDash

> Lightweight Git dashboard in system tray

常驻系统托盘的轻量级多仓库 Git 仪表盘，支持批量操作、实时状态监控和分组管理。

## 特性

- 🖥️ **系统托盘常驻** - 最小化到系统托盘，随时快速访问
- 📦 **多仓库管理** - 支持添加和管理多个 Git 仓库
- 📊 **实时状态监控** - 显示分支、ahead/behind、修改文件数等
- ⚡ **批量操作** - 支持批量 Pull/Fetch，并发控制
- 🎨 **分组管理** - 自定义分组，快速筛选项目
- 🌙 **暗色主题** - 专为开发者优化的暗色界面
- 🔍 **虚拟滚动** - 高性能渲染，支持数百个仓库

## 技术栈

### 前端
- **Vue 3** - 渐进式 JavaScript 框架
- **Pinia** - Vue 官方状态管理
- **TypeScript** - 类型安全的 JavaScript
- **TailwindCSS 4** - 实用优先 CSS 框架
- **Vite** - 下一代前端构建工具

### 后端
- **Tauri 2** - 轻量级桌面应用框架
- **Rust** - 安全、高性能的系统级语言
- **Tokio** - Rust 异步运行时
- **notify** - 跨平台文件监听

## 项目结构

```
gitdash/
├── src/                          # 前端源码
│   ├── assets/                   # 静态资源
│   │   └── tailwind.css          # TailwindCSS 样式
│   ├── components/               # Vue 组件
│   │   ├── Sidebar/              # 侧边栏组件
│   │   ├── ProjectList/          # 项目列表组件
│   │   ├── OperationPanel/       # 操作面板组件
│   │   ├── Modals/               # 弹窗组件
│   │   └── TitleBar.vue          # 标题栏
│   ├── composables/              # 组合式函数
│   │   └── useProjectStatus.ts   # 项目状态管理
│   ├── lib/                      # 工具库
│   │   └── tauriApi.ts           # Tauri API 封装
│   ├── stores/                   # Pinia 状态管理
│   │   ├── appStore.ts           # 应用主状态
│   │   └── operationStore.ts     # 操作队列状态
│   ├── types/                    # TypeScript 类型
│   │   └── index.ts              # 共享类型定义
│   ├── App.vue                   # 根组件
│   └── main.ts                   # 入口文件
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── main.rs               # 入口：初始化 + 托盘
│   │   ├── lib.rs                # 模块导出
│   │   ├── commands.rs           # Tauri Commands
│   │   ├── git.rs                # Git 执行器
│   │   ├── store.rs              # 配置读写 + 缓存
│   │   ├── watcher.rs            # 文件监听
│   │   ├── scanner.rs            # 项目扫描
│   │   └── models.rs             # 数据模型
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
├── index.html                    # HTML 入口
├── package.json                  # 项目配置
├── vite.config.ts                # Vite 配置
└── tsconfig.json                 # TypeScript 配置
```

## 开发指南

### 环境要求

- **Node.js** >= 18
- **pnpm** >= 8
- **Rust** >= 1.70
- **Git** >= 2.0

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# 首次运行需要安装 Rust 依赖（自动执行）
```

### 开发

```bash
# 启动开发服务器
pnpm dev
```

开发服务器会在 `http://localhost:1420` 启动，Tauri 窗口会自动打开。

### 构建

```bash
# 构建生产版本
pnpm build

# 构建可分发应用
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/`。

## 核心功能实现

### Git 执行器 (`git.rs`)

- 并发控制：使用 Tokio Semaphore 限制最大并发数（默认 3）
- 超时机制：单个 Git 命令最多 30 秒
- 状态解析：解析 `git status --porcelain -b` 输出
- 安全设置：`GIT_TERMINAL_PROMPT=0` 防止交互式提示

### 状态缓存 (`store.rs`)

- TTL 缓存：状态缓存 5 秒
- 自动失效：项目变化时清除缓存
- 配置文件：`%APPDATA%/GitDash/config.json` (Windows)

### 文件监听 (`watcher.rs`)

- 只监听 `.git/` 关键文件，避免性能问题
- 监听：`HEAD`, `index`, `refs/heads/`, `refs/remotes/`
- 变化时触发 `repo:changed` 事件到前端

### 虚拟滚动表格

- 自定义实现，不依赖第三方库
- 只渲染可视区域行
- 支持 Shift/Ctrl 多选

## 数据模型

### Project (项目)

```typescript
interface Project {
  id: string;           // UUID
  name: string;         // 仓库名称
  path: string;         // 绝对路径
  group_id: string | null;  // 分组 ID
  tags: string[];       // 标签
  is_favorite: boolean; // 是否收藏
  created_at: number;   // 创建时间戳
}
```

### ProjectStatus (项目状态)

```typescript
interface ProjectStatus {
  project_id: string;
  branch: string;       // 当前分支
  ahead: number;        // 超前远程提交数
  behind: number;       // 落后远程提交数
  modified: number;     // 修改文件数
  staged: number;       // 暂存文件数
  untracked: number;    // 未跟踪文件数
  is_clean: boolean;    // 是否干净
  last_fetched: number | null;
  is_fetching: boolean;
  error: string | null;
}
```

## Tauri Commands

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_config` | - | `AppConfig` | 获取完整配置 |
| `add_project` | `{ path: string }` | `Project` | 添加项目 |
| `remove_project` | `{ projectId: string }` | `void` | 删除项目 |
| `update_projects` | `{ projects: Project[] }` | `void` | 批量更新 |
| `get_project_status` | `{ projectId: string, force: boolean }` | `ProjectStatus` | 获取状态 |
| `batch_pull` | `{ projectIds: string[] }` | `GitResult[]` | 批量 Pull |
| `batch_fetch` | `{ projectIds: string[] }` | `GitResult[]` | 批量 Fetch |
| `open_repo_folder` | `{ projectId: string }` | `void` | 打开文件夹 |

## 事件系统

### Rust → 前端

| 事件名 | 载荷 | 说明 |
|--------|------|------|
| `git:progress` | `OperationEvent` | 批量操作进度 |
| `repo:changed` | `string` (projectId) | 仓库变化 |

## 配置说明

### tauri.conf.json

```json
{
  "app": {
    "windows": [{
      "decorations": false,  // 无边框窗口（自定义标题栏）
      "width": 1200,
      "height": 800
    }]
  },
  "plugins": {
    "fs": {
      "scope": {
        "allow": ["$APPDATA/**", "$HOME/**"]
      }
    }
  }
}
```

## 性能优化

- **安装包 < 10MB** - Tauri 相比 Electron 更轻量
- **内存 < 50MB** - 50 个仓库运行时
- **并发限制** - 最多 3 个并发 Git 操作
- **TTL 缓存** - 5 秒状态缓存减少重复查询
- **虚拟滚动** - 只渲染可视区域
- **精准监听** - 只监听 `.git/` 关键文件

## 开发计划

- [ ] 全局快捷键（打开/隐藏窗口）
- [ ] 拖拽排序
- [ ] 自定义主题
- [ ] 多语言支持
- [ ] 自动更新
- [ ] 插件系统

## 许可证

MIT License
