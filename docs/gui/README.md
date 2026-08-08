# litepack-gui 桌面应用文档

> 版本：v0.1.0
> 更新日期：2026-08-08

---

## 概述

`litepack-gui` 是基于 Tauri v2 + Vue 3 的跨平台桌面应用，提供图形化的压缩包管理界面。

## 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 后端 | Tauri | 2.11.5 |
| 前端框架 | Vue | 3.5.41 |
| 状态管理 | Pinia | 4.0.2 |
| 构建工具 | Vite | 8.2.1 |
| 类型检查 | TypeScript | ~6.0.3 |

## 开发环境

### 前置要求

- Rust >= 1.77.2
- Node.js >= 20.19
- pnpm 包管理器

### 启动开发

```bash
cd crates/litepack-gui
pnpm install
cargo tauri dev
```

### 构建发布

```bash
cd crates/litepack-gui
cargo tauri build
```

## 项目结构

```
crates/litepack-gui/
├── src/                    # Vue 3 前端
│   ├── api/               # Tauri IPC 封装
│   │   └── tauri.ts
│   ├── components/        # Vue 组件
│   │   ├── TitleBar.vue
│   │   ├── Toolbar.vue
│   │   ├── EmptyState.vue
│   │   ├── FileTable.vue
│   │   ├── StatusBar.vue
│   │   ├── ProgressBar.vue
│   │   ├── PasswordDialog.vue
│   │   └── SettingsMenu.vue
│   ├── stores/            # Pinia 状态
│   │   └── app.ts
│   ├── theme/             # 主题变量
│   │   └── variables.css
│   ├── types/             # TypeScript 类型
│   │   └── index.ts
│   ├── utils/             # 工具函数
│   │   └── format.ts
│   ├── App.vue
│   ├── main.ts
│   └── style.css
├── src-tauri/             # Tauri 后端
│   ├── src/
│   │   ├── lib.rs         # 命令注册
│   │   ├── commands.rs    # IPC 命令实现
│   │   ├── state.rs       # 状态管理
│   │   ├── context_menu.rs # 右键菜单
│   │   └── main.rs
│   ├── capabilities/      # Tauri 权限
│   ├── hooks.nsh          # NSIS 钩子
│   └── tauri.conf.json    # Tauri 配置
├── dist/                  # 构建产物
└── package.json
```

## 组件说明

### TitleBar - 标题栏

自定义窗口标题栏，包含：
- 应用图标和名称
- 归档文件名显示
- 解压后大小显示
- 最小化/最大化/关闭按钮

### Toolbar - 工具栏

操作按钮区域：
- **解压到** - 选择目录解压
- **一键解压** - 快速解压到归档所在目录
- **搜索** - 实时过滤文件列表
- **主题切换** - 亮色/暗色切换
- **设置** - 右键菜单开关

### EmptyState - 空状态

未打开归档时显示：
- 拖放区域
- "打开文件"按钮
- 文件选择对话框（过滤 .zip, .7z）

### FileTable - 文件表格

归档内容展示：
- 虚拟目录导航
- 面包屑路径
- 文件列表（名称、大小、压缩后、修改日期）
- 文件类型图标
- 加密条目锁图标
- 排序功能

### StatusBar - 状态栏

底部信息栏：
- 总大小 / 条目数
- 压缩率
- 归档格式

### ProgressBar - 进度条

操作进度显示：
- 操作阶段（列出/压缩/解压）
- 进度百分比
- 已完成/总字节数
- 取消按钮

### PasswordDialog - 密码对话框

模态对话框，用于：
- 打开加密 7z（头部加密）
- 解压加密文件
- 密码错误重试

### SettingsMenu - 设置菜单

弹出菜单：
- 右键菜单注册/注销开关

## 状态管理

### AppState (Pinia Store)

```typescript
interface AppState {
  theme: 'light' | 'dark';           // 主题
  archive: string;                    // 归档路径
  archiveName: string;                // 显示名称
  archiveFormat: 'zip' | '7z';       // 格式
  entries: EntryInfo[];               // 条目列表
  currentDir: string;                 // 当前目录
  search: string;                     // 搜索词
  selected: Set<string>;              // 选中项
  sortKey: 'name' | 'size' | 'modified';
  sortAsc: boolean;
  task: TaskState | null;             // 当前任务
}
```

### Getters

| Getter | 说明 |
|--------|------|
| `children` | 当前目录下的条目（过滤+排序） |
| `breadcrumbs` | 路径导航段 |
| `dirTotalSize` | 当前目录总大小 |
| `dirTotalCompressed` | 当前目录压缩后总大小 |

## 主题系统

### CSS 变量

```css
/* 亮色主题 */
[data-theme="light"] {
  --bg: #f8f9fa;
  --surface: #ffffff;
  --text: #212529;
  --accent: #0d6efd;
  --danger: #dc3545;
}

/* 暗色主题 */
[data-theme="dark"] {
  --bg: #1a1b1e;
  --surface: #25262b;
  --text: #c1c2c5;
  --accent: #339af0;
  --danger: #fa5252;
}
```

### 主题持久化

保存到 `localStorage` key `litepack-theme`。

## IPC 命令

### extract_archive

```typescript
invoke("extract_archive", {
  archive: string,      // 归档路径
  outDir: string,       // 输出目录
  password?: string,    // 密码
  overwrite: boolean,   // 覆盖模式
  progress: Channel,    // 进度通道
}): Promise<number>     // 返回操作 ID
```

### compress_files

```typescript
invoke("compress_files", {
  paths: string[],      // 源文件列表
  target: string,       // 输出路径
  format: string,       // 格式
  level: number,        // 压缩等级
  password?: string,    // 密码
  skipHidden: boolean,  // 跳过隐藏文件
  overwrite: boolean,   // 覆盖模式
  progress: Channel,    // 进度通道
}): Promise<number>
```

### list_archive

```typescript
invoke("list_archive", {
  archive: string,      // 归档路径
  password?: string,    // 密码
}): Promise<EntryInfo[]>
```

### cancel_operation

```typescript
invoke("cancel_operation", {
  id: number,           // 操作 ID
}): Promise<void>
```

## 窗口配置

| 属性 | 值 |
|------|-----|
| 默认尺寸 | 960 × 700 px |
| 最小尺寸 | 720 × 520 px |
| 自定义标题栏 | 是 |
| 拖放支持 | 是 |
| 标识符 | com.litepack.app |

## Windows 右键菜单

### 注册表位置

- `HKCU\Software\Classes\SystemFileAssociations\.zip\shell\LitePackExtract`
- `HKCU\Software\Classes\SystemFileAssociations\.7z\shell\LitePackExtract`

### 注册/注销

通过设置菜单的开关控制，或使用 Tauri 命令：

```typescript
// 注册
invoke("register_context_menu")

// 注销
invoke("unregister_context_menu")

// 查询状态
invoke("context_menu_status"): Promise<boolean>
```

## 交互流程

### 打开归档

```
1. 双击 .zip/.7z 文件（系统关联）
   或 拖放文件到窗口
   或 点击"打开"按钮
2. 调用 list_archive
3. 渲染文件列表
4. 显示状态栏信息
```

### 解压文件

```
1. 点击"一键解压"或"解压到"
2. 计算输出目录
3. 调用 extract_archive
4. 显示进度条
5. 完成后显示 toast
```

### 密码处理

```
场景 1: 打开加密 7z
1. list_archive → 密码错误
2. 弹出密码对话框
3. 用户输入密码
4. 重新调用 list_archive

场景 2: 解压加密文件
1. extract_archive → 密码错误
2. 弹出密码对话框
3. 用户输入密码
4. 重新调用 extract_archive
```

## 系统要求

### Windows

- Windows 10+
- WebView2 运行时

### Linux

- libwebkit2gtk-4.1-dev
- libappindicator3-dev
- librsvg2-dev

### macOS

- macOS 11.0+
- Xcode Command Line Tools

## 调试

### 开发模式

```bash
# 启动开发服务器（带热重载）
cargo tauri dev

# 仅启动前端
pnpm dev

# 仅启动后端
cd src-tauri && cargo run
```

### 日志

Tauri 后端日志通过 `eprintln!` 输出到 stderr，在终端运行时可见。

---

> 文档结束
