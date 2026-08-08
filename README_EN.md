<div align="center">

# LitePack

**Lightweight Cross-Platform Archive Manager**

ZIP / 7z Compress · Extract · Browse · Context Menu

[![CI](https://github.com/user/litepack/actions/workflows/ci.yml/badge.svg)](https://github.com/user/litepack/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.77+-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-v2-green.svg)](https://tauri.app)

</div>

---

[中文](README.md)

## Features

### Archive Formats

| Format | Compress | Extract | Browse | Encrypt |
|--------|:--------:|:-------:|:------:|:-------:|
| **ZIP** | ✅ | ✅ | ✅ | AES-256 |
| **7z** | ✅ | ✅ | ✅ | AES-256 |

> RAR is not supported (RARLAB license prohibits free redistribution)

### GUI Desktop App

- **Archive Browser** — Virtual directory navigation, breadcrumb path, real-time search
- **Quick Extract** — One-click extract to archive location
- **Encryption Support** — Auto-detect encrypted entries, password dialog
- **Theme Switching** — Light / Dark themes
- **Progress Display** — Real-time progress bar, file-level tracking
- **Drag & Drop** — Drag files directly to open
- **Context Menu** — Windows Explorer right-click integration

### CLI Tool

```
litepack pack <source>... -o <output.7z|output.zip> [OPTIONS]
litepack unpack <archive> [OPTIONS]
litepack list <archive>
```

- Compression levels 0-9
- Progress bar with ETA
- Ctrl+C cancellation

### Security

- **Path traversal protection** — Rejects `..`, absolute paths, drive letter prefixes
- **Zip bomb detection** — Max 100,000 entries / 256 GiB / 10,000:1 ratio
- **Password error distinction** — Clear messages for "wrong password" vs "password required"

## Tech Stack

| Layer | Technology |
|-------|------------|
| Core Library | Rust · `zip` · `sevenz-rust2` |
| CLI | Rust · `clap` · `indicatif` |
| GUI Backend | Tauri v2 · Rust |
| GUI Frontend | Vue 3 · TypeScript · Vite · Pinia |
| Installer | NSIS (Windows) · AppImage/Deb (Linux) · DMG (macOS) |

## Project Structure

```
LitePack/
├── crates/
│   ├── litepack-core/     # Pure Rust compression core library
│   ├── litepack-cli/      # CLI command-line tool
│   └── litepack-gui/      # Tauri v2 desktop app
│       ├── src/           # Vue 3 frontend
│       └── src-tauri/     # Tauri v2 backend
├── .github/workflows/     # CI/CD
└── LICENSE                # MIT License
```

## Dependencies

### litepack-core

| Dependency | Version | Description |
|------------|---------|-------------|
| `zip` | 8.6.0 | ZIP format compress/extract |
| `sevenz-rust2` | 0.21.4 | 7z format compress/extract |
| `serde` | 1.0.229 | Serialization framework |
| `serde_json` | 1.0.151 | JSON support |
| `thiserror` | 2.0.19 | Error type derive macro |
| `walkdir` | 2.5.0 | Recursive directory traversal |
| `rayon` | 1.12.0 | Parallel file processing |
| `parking_lot` | 0.12.5 | High-performance locks |

### litepack-cli

| Dependency | Version | Description |
|------------|---------|-------------|
| `clap` | 4.6.6 | CLI argument parsing |
| `indicatif` | 0.18.6 | Terminal progress bars |
| `anyhow` | 1.0.104 | Error handling |
| `ctrlc` | 3.4 | Ctrl+C signal handling |

### litepack-gui

| Dependency | Version | Description |
|------------|---------|-------------|
| `tauri` | 2.11.5 | Desktop app framework |
| `tauri-plugin-dialog` | 2.7.2 | Native file picker dialog |
| `tauri-plugin-fs` | 2.5.1 | Filesystem access plugin |
| `tauri-build` | 2.6.3 | Tauri build tool (build-only) |
| `winreg` | 0.55.0 | Windows registry access (Windows only) |
| `windows-sys` | 0.60.0 | Windows API bindings (Windows only) |

## Getting Started

### Prerequisites

- **Rust** >= 1.77.2
- **Node.js** >= 20.19 + **pnpm**
- **Windows**: MSVC toolchain + WebView2
- **Linux**: `libwebkit2gtk-4.1-dev` `libappindicator3-dev` `librsvg2-dev`

### Development

```bash
# GUI dev mode
cd crates/litepack-gui
pnpm install
cargo tauri dev

# CLI build
cargo build --release -p litepack-cli

# Run tests
cargo test --workspace
```

### Building Release

```bash
cd crates/litepack-gui
cargo tauri build
```

Artifacts are in `target/release/bundle/`.

## CLI Examples

```bash
# Compress files
litepack pack src/ docs/ -o backup.zip --level 6

# Password-protected compression
litepack pack secret.txt -o encrypted.7z --password mypass

# Extract
litepack unpack archive.zip -d ./output

# List contents
litepack list archive.7z
```

## License

This project is licensed under the [MIT License](LICENSE).

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for all dependency license information.
