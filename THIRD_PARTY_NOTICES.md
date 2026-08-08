# THIRD PARTY NOTICES

LitePack 依赖以下第三方开源软件。各组件遵循其原始许可证。完整文本见各 crate 源码或 [SPDX](https://spdx.org/licenses/)。

## Rust 直接依赖

| 组件 | 版本 | 许可证 | 说明 |
|---|---|---|---|
| thiserror | 2.0.19 | MIT OR Apache-2.0 | 错误派生宏 |
| serde | 1.0.229 | MIT OR Apache-2.0 | 序列化框架 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | JSON 序列化 |
| walkdir | 2.5.0 | Unlicense/MIT | 目录遍历 |
| zip | 8.6.0 | MIT | ZIP 读写 |
| sevenz-rust2 | 0.21.4 | Apache-2.0 | 7z 读写 |
| rayon | 1.12.0 | MIT OR Apache-2.0 | 并行迭代器 |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | 高性能锁 |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | 错误处理 |
| clap | 4.6.6 | MIT OR Apache-2.0 | CLI 参数解析 |
| indicatif | 0.18.6 | MIT | 进度条 |
| path-slash | 0.2.1 | MIT | 路径分隔符统一 |
| urlencoding | 2.1.3 | MIT | URL 编码 |
| log | 0.4 | MIT OR Apache-2.0 | 日志门面 |
| tauri | 2.11.5 | Apache-2.0 OR MIT | 桌面框架 |
| tauri-build | 2.6.3 | Apache-2.0 OR MIT | 构建工具 |
| tauri-plugin-dialog | 2.7.2 | Apache-2.0 OR MIT | 对话框插件 |
| tauri-plugin-fs | 2.5.1 | Apache-2.0 OR MIT | 文件系统插件 |
| tauri-plugin-log | 2 | Apache-2.0 OR MIT | 日志插件 |
| winreg | 0.55.0 | MIT | Windows 注册表 |
| windows-sys | 0.60.0 | MIT OR Apache-2.0 | Windows API |
| ctrlc | 3.4 | MIT/Apache-2.0 | Ctrl+C 处理 |

## 前端依赖

前端使用 Vue 3、Vite 8、TypeScript、Pinia、vue-router、@tauri-apps/api 及 Tauri 官方插件（dialog/fs/log）。

## 运行库说明

- **zip**（ZIP 读写）、**sevenz-rust2**（7z 读写）与 **walkdir**（目录遍历）为压缩核心。
- **tauri** 桌面框架按 LGPL/MIT 双许可分发，随安装包分发二进制时遵守其许可条款。
- **indicatif**、**clap**、**anyhow**、**thiserror** 为 CLI 基础设施。
- **path-slash** 用于统一 Windows/Unix 路径分隔符。
- **tauri-plugin-log** 提供日志输出，支持文件轮转。

本清单仅列出 Cargo.toml 中的直接依赖，不含传递依赖。
