# litepack-core 核心库文档

> 版本：v0.1.0
> 更新日期：2026-08-08

---

## 概述

`litepack-core` 是纯 Rust 实现的压缩核心库，提供 ZIP 和 7z 格式的压缩、解压、浏览功能。供 CLI 和 GUI 应用调用。

## 模块结构

```
crates/litepack-core/src/
├── lib.rs          # 入口，公开 API
├── entry.rs        # 归档条目数据结构
├── error.rs        # 错误类型定义
├── format.rs       # 格式检测
├── options.rs      # 压缩/解压选项
├── progress.rs     # 进度回调
├── walk.rs         # 文件遍历
├── zip.rs          # ZIP 格式实现
└── sevenz.rs       # 7z 格式实现
```

## 核心 API

### 格式检测

```rust
pub fn detect_format(path: &Path) -> Result<ArchiveFormat>
```

通过魔数字节检测格式（ZIP: `PK`, 7z: `37 7A BC AF 27 1C`），回退到扩展名。

### 列出内容

```rust
pub fn list(archive: &Path) -> Result<Vec<EntryInfo>>
pub fn list_with_password(archive: &Path, password: Option<&str>) -> Result<Vec<EntryInfo>>
```

### 压缩

```rust
pub fn compress(
    entries: &[CompressEntry],
    out: &Path,
    opts: &CompressOptions,
    sink: &dyn ProgressSink,
    cancel: &CancelHandle,
) -> Result<()>
```

### 解压

```rust
pub fn decompress(
    archive: &Path,
    out_dir: &Path,
    opts: &ExtractOptions,
    sink: &dyn ProgressSink,
    cancel: &CancelHandle,
) -> Result<()>
```

## 数据结构

### EntryInfo

```rust
pub struct EntryInfo {
    pub path: String,                    // 条目路径
    pub size: u64,                       // 解压后大小
    pub is_dir: bool,                    // 是否为目录
    pub compressed_size: Option<u64>,    // 压缩后大小
    pub method: Option<String>,          // 压缩方法
    pub modified: Option<i64>,           // 修改时间
    pub encrypted: bool,                 // 是否加密
}
```

### ProgressReport

```rust
pub struct ProgressReport {
    pub phase: Phase,           // 操作阶段
    pub current_file: PathBuf,  // 当前处理文件
    pub done_bytes: u64,        // 已完成字节数
    pub total_bytes: u64,       // 总字节数
}
```

### Options

```rust
pub struct ExtractOptions {
    pub password: Option<String>,
    pub overwrite: OverwriteMode,
}

pub struct CompressOptions {
    pub level: u8,                    // 0-9
    pub password: Option<String>,
    pub overwrite: bool,
    pub skip_hidden: bool,
}
```

## 安全机制

### 路径穿越防护

`safe_join()` 验证所有条目路径，拒绝 `..`、绝对路径、盘符前缀。

```rust
pub(crate) fn safe_join(out_dir: &Path, name: &str) -> Result<PathBuf>
```

### Zip 炸弹检测

| 限制 | 值 |
|------|-----|
| 最大条目数 | 100,000 |
| 最大解压后总大小 | 256 GiB |
| 最大压缩比 | 10,000:1 |

### 错误类型

```rust
pub enum Error {
    Io(std::io::Error),
    UnsupportedFormat(String),
    BadPassword,
    Encrypted,
    PathTraversal(String),
    ZipBomb,
    Cancelled,
}
```

## 格式支持

### ZIP

| 特性 | 支持 |
|------|------|
| Deflated 压缩 | ✅ |
| Stored (不压缩) | ✅ |
| AES-256 加密 | ✅ |
| 修改时间保留 | ✅ |
| 压缩等级 0-9 | ✅ |

### 7z

| 特性 | 支持 |
|------|------|
| LZMA2 压缩 | ✅ |
| COPY (不压缩) | ✅ |
| AES-256 加密 | ✅ |
| 头部加密 | ✅ |
| DEFLATE 编解码器 | ✅ |
| LZ4 编解码器 | ✅ |
| ZSTD 编解码器 | ✅ |

## 依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `zip` | 8.6.0 | ZIP 格式支持 |
| `sevenz-rust2` | 0.21.4 | 7z 格式支持 |
| `serde` | 1.0.229 | 序列化 |
| `serde_json` | 1.0.151 | JSON 支持 |
| `thiserror` | 2.0.19 | 错误派生宏 |
| `walkdir` | 2.5.0 | 目录遍历 |
| `rayon` | 1.12.0 | 并行处理 |
| `parking_lot` | 0.12.5 | 同步原语 |

## 测试

```bash
# 运行所有核心库测试
cargo test -p litepack-core

# 运行特定测试
cargo test -p litepack-core -- zip
cargo test -p litepack-core -- sevenz
cargo test -p litepack-core -- format
```

### 测试覆盖

| 模块 | 测试用例 |
|------|----------|
| format.rs | 魔数检测、扩展名回退、未知格式拒绝 |
| zip.rs | 压缩/解压往返、密码往返、路径穿越拒绝、炸弹检测、取消操作 |
| sevenz.rs | 压缩/解压往返、密码往返、路径穿越拒绝、取消操作 |
| walk.rs | 文件收集、隐藏文件跳过、单文件基名 |

---

> 文档结束
