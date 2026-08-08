# LitePack 开发计划

跨平台压缩/解压工具。架构：`litepack-core`（纯 Rust 核心库）→ 两个入口：`litepack-cli`（命令行）与 `litepack-gui`（Tauri v2 桌面应用，Vue3 + TS 前端）。仅支持 ZIP 与 7z。全部依赖 MIT / Apache-2.0 许可证合规，纯 Rust 编解码。

## 1. 技术栈与版本（全部为最新稳定版）

### Rust 依赖

| crate | 版本 | 许可证 | 用途 |
|---|---|---|---|
| `zip` | 8.6.0 | MIT | ZIP 编解码 |
| `sevenz-rust2` | 0.21.4 | Apache-2.0 | 7z 编解码（配 `default-features=false` + `compress,deflate,lz4,zstd`） |
| `tauri` | 2.11.5 | Apache-2.0 OR MIT | GUI 框架 |
| `tauri-build` | 2.6.3 | Apache-2.0 OR MIT | Tauri 构建脚本 |
| `tauri-plugin-dialog` | 2.7.2 | Apache-2.0 OR MIT | 文件对话框 |
| `tauri-plugin-fs` | 2.5.1 | Apache-2.0 OR MIT | 文件系统 |
| `clap` | 4.6.6 | MIT OR Apache-2.0 | CLI 参数 |
| `indicatif` | 0.18.6 | MIT | CLI 进度条 |
| `thiserror` | 2.0.19 | MIT OR Apache-2.0 | 错误枚举 |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | CLI 错误处理 |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | 序列化 / IPC |
| `serde_json` | 1.0.151 | MIT OR Apache-2.0 | IPC 数据 |
| `walkdir` | 2.5.0 | Unlicense/MIT | 目录递归 |
| `rayon` | 1.12.0 | MIT OR Apache-2.0 | 并行压缩（可选） |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | 互斥锁 |

### 前端依赖（npm）

| 包 | 版本 | 许可证 | 用途 |
|---|---|---|---|
| `vue` | 3.5.41 | MIT | UI 框架 |
| `typescript` | `~6.0`（6.0.3） | Apache-2.0 | 类型检查（锁定 6.x，不用 TS 7） |
| `vite` | 8.2.1 | MIT | 构建工具（需 Node ≥ 20.19） |
| `@vitejs/plugin-vue` | 6.0.8 | MIT | Vue 插件 |
| `@tauri-apps/api` | 2.11.1 | Apache-2.0 OR MIT | IPC 调用 |
| `@tauri-apps/plugin-dialog` | 2.7.2 | MIT OR Apache-2.0 | 对话框 |
| `@tauri-apps/plugin-fs` | 2.5.1 | MIT OR Apache-2.0 | 文件系统 |
| `pinia` | 4.0.2 | MIT | 状态管理 |
| `vue-tsc` | 3.3.9 | MIT | Vue 类型检查 |

### 构建工具

- `tauri-cli 2.11.4`（Rust 版，`cargo install tauri-cli --version 2.11.4 --locked`）
- 前端**不**引入 `@tauri-apps/cli` npm 包，所有 Tauri 操作走 `cargo tauri ...`

## 2. 目录结构

```
LitePack/
├── Cargo.toml                    # [workspace] members: core, cli, gui/src-tauri
├── package.json / pnpm-lock.yaml
├── LICENSE-MIT / LICENSE-APACHE
├── THIRD_PARTY_NOTICES.md        # 第三方声明（sevenz-rust2、zstd BSD 等）
├── .github/workflows/ci.yml
├── .opencode/
│   ├── PLAN.md                   # 本计划文件
│   └── command/plan.md           # /plan 命令
├── crates/
│   ├── litepack-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # 统一入口 API
│   │       ├── format.rs         # 魔数/扩展名识别
│   │       ├── error.rs          # Error 枚举
│   │       ├── options.rs        # CompressOptions/ExtractOptions
│   │       ├── progress.rs       # ProgressReport/Sink/CancelHandle
│   │       ├── entry.rs          # EntryInfo
│   │       ├── walk.rs           # 目录→条目列表
│   │       ├── zip.rs            # ZIP 编解码
│   │       └── sevenz.rs         # 7z 编解码
│   ├── litepack-cli/
│   │   ├── Cargo.toml
│   │   └── src/main.rs           # clap + indicatif
│   └── litepack-gui/
│       ├── package.json / vite.config.ts / tsconfig.json / index.html
│       ├── src/                  # Vue3 前端
│       │   ├── main.ts / App.vue
│       │   ├── api/tauri.ts      # invoke 封装 + 类型
│       │   ├── stores/app.ts     # pinia
│       │   ├── types/index.ts
│       │   ├── theme/            # CSS 变量设计系统
│       │   └── components/
│       │       ├── CompressPanel.vue / ExtractPanel.vue
│       │       ├── ArchiveList.vue / ProgressOverlay.vue
│       │       ├── FileDropZone.vue / OptionBar.vue
│       └── src-tauri/
│           ├── Cargo.toml / build.rs / tauri.conf.json
│           ├── icons/
│           └── src/
│               ├── main.rs       # 注册命令/插件
│               ├── commands.rs   # IPC 命令
│               └── state.rs      # 操作注册表（取消句柄）
```

## 3. 核心库 `litepack-core` API

```rust
pub enum ArchiveFormat { Zip, SevenZ }

pub struct EntryInfo { path: String, size: u64, is_dir: bool, compressed_size: Option<u64>, method: Option<String> }
pub struct CompressOptions { level: u32, password: Option<String>, overwrite: bool, skip_hidden: bool }
pub struct ExtractOptions { password: Option<String>, overwrite: OverwriteMode }
pub struct ProgressReport { phase: Phase, current_file: PathBuf, done_bytes: u64, total_bytes: u64 }

pub trait ProgressSink: Send + Sync { fn update(&self, r: ProgressReport); }
pub struct CancelHandle(Arc<AtomicBool>);

pub fn detect_format(path: &Path) -> Result<ArchiveFormat>;          // 魔数 50 4B / 37 7A BC，扩展名兜底
pub fn list(archive: &Path) -> Result<Vec<EntryInfo>>;
pub fn compress(entries: &[PathBuf], out: &Path, opts, sink, cancel) -> Result<()>;
pub fn decompress(archive: &Path, out_dir: &Path, opts, sink, cancel) -> Result<()>;
```

- 错误枚举（`thiserror`）：`Io` / `UnsupportedFormat` / `BadPassword` / `PathTraversal` / `ZipBomb` / `Cancelled` / `Encrypted`
- 7z 侧：`sevenz-rust2`，加密用 aes256 特性（`ArchiveWriter` + `AesEncoderOptions`）
- ZIP 侧：`zip`，默认 deflate，支持密码（ZipCrypto/AES）

## 4. CLI `litepack-cli`

```
litepack pack <src>... -o out.zip|out.7z [--level 1-9] [--password P] [--overwrite] [--skip-hidden]
litepack unpack <archive> [-d outdir] [--password P] [--overwrite]
litepack list <archive>
```

- 扩展名自动推断格式，`--format` 强制指定
- `indicatif` 进度条由 `ProgressSink` 驱动；Ctrl+C 触发 `CancelHandle`
- 退出码：0 成功 / 1 用户取消 / 2 其他错误

## 5. Tauri 后端（`src-tauri`）

IPC 命令（`#[tauri::command]`）：

```rust
async fn compress_files(app, paths: Vec<String>, target: String, format: String,
                        level: u32, password: Option<String>, progress: Channel<ProgressReport>) -> Result<u64, String>
async fn extract_archive(app, archive: String, out_dir: String, password: Option<String>,
                         progress: Channel<ProgressReport>) -> Result<u64, String>
async fn list_archive(archive: String) -> Result<Vec<EntryInfo>, String>
async fn cancel_operation(id: u64) -> Result<(), String>
```

- 进度/取消：每次操作注册 `(u64 id, CancelHandle)` 到 `AppState`（`Mutex<HashMap>`）；`tauri::ipc::Channel` 流式推送 `ProgressReport`；前端点取消 → `cancel_operation(id)` 置取消标志
- 线程：`async_runtime::spawn_blocking` 跑核心库阻塞 IO
- 文件对话框：`tauri-plugin-dialog`；拖拽：`onDragDropEvent`
- `tauri.conf.json`：`identifier: com.litepack.app`、`frontendDist: ../dist`、`beforeDevCommand: pnpm dev`、`bundle.targets: ["nsis"]`（Windows）

## 6. Vue3 前端（按设计图 v2：归档浏览器）

- 界面参考 `litepack-design-v2.html`：自定义标题栏（无边框 `decorations:false` + 拖拽/最小化/最大化/关闭）、工具栏（解压到/一键解压/搜索/主题）、文件表格（名称/压缩前/压缩后/类型/修改日期，排序+选择）、面包屑导航、状态栏（大小/数量/压缩率/格式）、底部进度条
- 打开归档：空态拖拽或对话框 → `list_archive` → 前端按路径构造目录树，进入目录导航
- 解压：`extract_archive` 到所选目录；加密归档弹出密码对话框（Encrypted/BadPassword 事件触发重试）
- pinia 管理状态：archive/entries/currentDir/search/selected/task
- 主题：CSS 变量设计系统 `theme/variables.css`，`[data-theme]` 切换并持久化 localStorage
- 待办（后续里程碑）：压缩界面（新建归档）、往归档添加条目、从归档删除条目

## 7. 安全设计

| 风险 | 对策 |
|---|---|
| 目录穿越 | 路径组件校验，拒绝 `..` 与绝对路径 |
| Zip 炸弹 | 比例/条目数/总量三上限 |
| 加密归档 | 密码错误明确报错；7z 用 AES-256 |
| 符号链接 | 压缩可选跟随，解压不写链接目标越界 |
| 取消 | 原子标志轮询，操作内安全中断 |

## 8. 测试计划

- 单元：魔数识别、路径校验、选项序列化
- Round-trip：ZIP/7z 压缩→解压字节级一致；空目录/大文件/中文文件名/深路径
- 互操作：`std::process` 调系统 `7z`/Python `zipfile` 交叉验证（存在则跑）
- 安全：构造恶意 `../` 归档、超高比炸弹归档断言拒绝
- 前端：`vue-tsc` 类型检查；`cargo test` + `cargo clippy -- -D warnings` 门禁

## 9. 打包与发布

- Windows：NSIS 安装器（含 WebView2 引导）；后续可加 MSI/签名
- macOS：`.dmg`；Linux：`.deb`（避免 AppImage 的 GStreamer GPL 插件问题）
- CI（GitHub Actions）：windows/ubuntu/macos 三平台 `cargo build+test`、`pnpm build`、`tauri build`

## 10. 里程碑（勾选更新进度）

- [x] **M0 脚手架**：workspace + `litepack-core` 骨架（format/error/options/progress）+ 魔数识别单测
- [x] **M1 ZIP**：`zip.rs` 压缩/解压/list + round-trip 测试
- [x] **M2 7z**：`sevenz.rs` 压缩/解压/list + 互操作测试
- [x] **M3 CLI**：pack/unpack/list + 进度条 + 取消 + 退出码
- [x] **M4 Tauri 工程**：用 `tauri-cli` 初始化（见第 11 节流程）
- [x] **M5 前端**：三视图 + 拖拽 + 进度/取消 + 主题系统（亮/暗）；验证 TS 6 + vue-tsc 3.3.9 + Vite 8 兼容性
- [x] **M6 硬化**：安全用例、错误处理打磨、`clippy -- -D warnings`
- [x] **M7 发布**：LICENSE/THIRD_PARTY_NOTICES、NSIS 打包、CI 三平台
- [x] **M8 右键菜单**：Windows 资源管理器 .zip/.7z 右键「直接解压」（HKCU 注册 + `--extract-here` 自动解压后退出 + 设置开关 + NSIS 安装/卸载钩子）

## 11. Tauri 工程初始化流程（M4）

1. 安装：`cargo install tauri-cli --version 2.11.4 --locked`
2. 前端脚手架：在 `crates/litepack-gui/` 用 Vite 生成 Vue-TS 项目（或手写 `package.json` + `vite.config.ts` + `src/`）
3. 初始化：`cargo tauri init`（在 `litepack-gui/` 执行），配置应用名 `LitePack`、`identifier com.litepack.app`、前端资源路径 `../dist`、dev 地址 `http://localhost:5173`
4. 接线 `tauri.conf.json`：`frontendDist: ../dist`、`devUrl: http://localhost:5173`、`beforeDevCommand: pnpm dev`、`beforeBuildCommand: pnpm build`
5. 加入 workspace：`Cargo.toml` 成员追加 `crates/litepack-gui/src-tauri`

日常命令：`cargo tauri dev`（开发）/ `cargo tauri build`（打包）；图标 `cargo tauri icon`。

## 12. 环境要求

- Rust stable ≥ 1.93（`sevenz-rust2` MSRV）
- Node.js ≥ 20.19（Vite 8）+ pnpm
- Windows：MSVC 工具链 + WebView2（Win11 自带）

## 13. 风险与对策

| 风险 | 对策 |
|---|---|
| `sevenz-rust2` API 演进快 | 锁 `=0.21`，封装在 `sevenz.rs` 隔离变更 |
| 7z 无 bzip2/PPMd 方法（纯 Rust 约束） | 仅 LZMA2/DEFLATE/LZ4/ZSTD，文档注明 |
| `zip` 9.0-pre 不稳 | 固定 8.x 稳定版 |
| TS 6 + vue-tsc 3.3.9 兼容 | M5 验证；有问题锁 TS `~5.9` |
| Tauri 首次构建慢 | 文档提示，CI 缓存 cargo/npm 依赖 |

## 14. 决策记录

- 支持格式：仅 ZIP + 7z
- GUI：Tauri v2 + Vue3 + TS（ts 锁定 `~6.0`，创建使用 `tauri-cli`）
- 依赖：全最新稳定版，许可证 MIT/Apache-2.0
- 纯 Rust 编解码，不用 `unrar`（RARLAB 许可证禁止自由分发）
- 加密：做（7z AES-256 + ZIP 密码）；主题：亮/暗两套；状态管理：pinia；包管理：pnpm
- 右键菜单：仅 Windows，注册在 `HKCU\Software\Classes\.zip|.7z\shell\LitePackExtractHere`（winreg 0.55，Windows-only target 依赖）；`--extract-here <path>` 启动参数由前端 `take_pending_extract()` 读取并自动解压到归档所在目录、完成后 `exit_app()` 退出；NSIS 用 `installerHooks`（hooks.nsh）在安装时写键、卸载时删键
