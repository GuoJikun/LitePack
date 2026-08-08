<div align="center">

# LitePack

**轻量级跨平台压缩包管理工具**

ZIP / 7z 压缩 · 解压 · 浏览 · 右键集成

[![CI](https://github.com/user/litepack/actions/workflows/ci.yml/badge.svg)](https://github.com/user/litepack/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.77+-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-v2-green.svg)](https://tauri.app)

</div>

---

[English](README_EN.md)

## 功能特性

### 压缩格式

| 格式 | 压缩 | 解压 | 浏览 | 加密 |
|------|:----:|:----:|:----:|:----:|
| **ZIP** | ✅ | ✅ | ✅ | AES-256 |
| **7z** | ✅ | ✅ | ✅ | AES-256 |

> 不支持 RAR 格式（RARLAB 许可证禁止免费分发）

### GUI 桌面应用

- **归档浏览器** — 虚拟目录导航，面包屑路径，实时搜索
- **一键解压** — 快速解压到归档所在目录
- **加密支持** — 自动检测加密条目，密码输入框
- **主题切换** — 亮色 / 暗色主题
- **进度显示** — 实时进度条，文件级进度跟踪
- **拖放支持** — 直接拖入文件打开
- **右键集成** — Windows 资源管理器右键菜单

### CLI 命令行工具

```
litepack pack <源文件>... -o <输出.7z|输出.zip> [选项]
litepack unpack <归档文件> [选项]
litepack list <归档文件>
```

- 压缩等级 0-9
- 进度条 + ETA 显示
- Ctrl+C 取消操作

### 安全特性

- **路径穿越防护** — 拒绝 `..`、绝对路径、盘符前缀
- **Zip 炸弹检测** — 最大 100,000 条目 / 256 GiB / 10,000:1 压缩比
- **密码错误区分** — 明确提示"密码错误"与"需要密码"

## 技术栈

| 层级 | 技术 |
|------|------|
| 核心库 | Rust · `zip` · `sevenz-rust2` |
| CLI | Rust · `clap` · `indicatif` |
| GUI 后端 | Tauri v2 · Rust |
| GUI 前端 | Vue 3 · TypeScript · Vite · Pinia |
| 安装包 | NSIS (Windows) · AppImage/Deb (Linux) · DMG (macOS) |

## 项目结构

```
LitePack/
├── crates/
│   ├── litepack-core/     # 纯 Rust 压缩核心库
│   ├── litepack-cli/      # CLI 命令行工具
│   └── litepack-gui/      # Tauri v2 桌面应用
│       ├── src/           # Vue 3 前端
│       └── src-tauri/     # Tauri v2 后端
├── .github/workflows/     # CI/CD
└── LICENSE                # MIT License
```

## 依赖

### litepack-core

| 依赖 | 版本 | 说明 |
|------|------|------|
| `zip` | 8.6.0 | ZIP 格式压缩/解压 |
| `sevenz-rust2` | 0.21.4 | 7z 格式压缩/解压 |
| `serde` | 1.0.229 | 序列化框架 |
| `serde_json` | 1.0.151 | JSON 支持 |
| `thiserror` | 2.0.19 | 错误类型派生宏 |
| `walkdir` | 2.5.0 | 递归目录遍历 |
| `rayon` | 1.12.0 | 并行文件处理 |
| `parking_lot` | 0.12.5 | 高性能锁 |

### litepack-cli

| 依赖 | 版本 | 说明 |
|------|------|------|
| `clap` | 4.6.6 | CLI 参数解析 |
| `indicatif` | 0.18.6 | 终端进度条 |
| `anyhow` | 1.0.104 | 错误处理 |
| `ctrlc` | 3.4 | Ctrl+C 信号处理 |

### litepack-gui

| 依赖 | 版本 | 说明 |
|------|------|------|
| `tauri` | 2.11.5 | 桌面应用框架 |
| `tauri-plugin-dialog` | 2.7.2 | 原生文件选择对话框 |
| `tauri-plugin-fs` | 2.5.1 | 文件系统访问插件 |
| `tauri-build` | 2.6.3 | Tauri 构建工具（仅构建时） |
| `winreg` | 0.55.0 | Windows 注册表访问（仅 Windows） |
| `windows-sys` | 0.60.0 | Windows API 绑定（仅 Windows） |

## 快速开始

### 环境要求

- **Rust** >= 1.77.2
- **Node.js** >= 20.19 + **pnpm**
- **Windows**: MSVC 工具链 + WebView2
- **Linux**: `libwebkit2gtk-4.1-dev` `libappindicator3-dev` `librsvg2-dev`

### 开发

```bash
# GUI 开发模式
cd crates/litepack-gui
pnpm install
cargo tauri dev

# CLI 构建
cargo build --release -p litepack-cli

# 运行测试
cargo test --workspace
```

### 构建发布

```bash
cd crates/litepack-gui
cargo tauri build
```

产物位于 `target/release/bundle/`。

## CLI 使用示例

```bash
# 压缩文件
litepack pack src/ docs/ -o backup.zip --level 6

# 带密码压缩
litepack pack secret.txt -o encrypted.7z --password mypass

# 解压
litepack unpack archive.zip -d ./output

# 查看内容
litepack list archive.7z
```

## 开源协议

本项目采用 [MIT License](LICENSE) 开源。

详见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) 了解所有依赖的许可证信息。
