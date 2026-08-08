# litepack-cli 命令行工具文档

> 版本：v0.1.0
> 更新日期：2026-08-08

---

## 概述

`litepack-cli` 是 LitePack 的命令行工具，提供压缩、解压、列出内容功能，支持进度条显示和 Ctrl+C 取消。

## 安装

```bash
# 从源码构建
cargo build --release -p litepack-cli

# 产物位置
target/release/litepack.exe    # Windows
target/release/litepack        # Linux/macOS
```

## 命令

### pack - 压缩文件

```bash
litepack pack <source>... -o <output> [OPTIONS]
```

**参数：**

| 参数 | 说明 | 必填 | 默认值 |
|------|------|:----:|--------|
| `<source>...` | 源文件/文件夹路径 | 是 | - |
| `-o, --output` | 输出文件路径 | 是 | - |

**选项：**

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--level <0-9>` | 压缩等级 | 6 |
| `--password <P>` | 加密密码 | 无 |
| `--format <zip\|7z>` | 强制格式 | 自动检测 |
| `--overwrite` | 覆盖现有文件 | 否 |
| `--skip-hidden` | 跳过隐藏文件 | 否 |

**示例：**

```bash
# 压缩文件夹
litepack pack src/ docs/ -o backup.zip

# 指定压缩等级
litepack pack large_file.iso -o archive.7z --level 9

# 带密码压缩
litepack pack secret.txt -o encrypted.7z --password mypass

# 强制 7z 格式
litepack pack data/ -o output.7z --format 7z

# 覆盖已存在文件
litepack pack files/ -o backup.zip --overwrite
```

### unpack - 解压压缩包

```bash
litepack unpack <archive> [OPTIONS]
```

**参数：**

| 参数 | 说明 | 必填 | 默认值 |
|------|------|:----:|--------|
| `<archive>` | 压缩包路径 | 是 | - |

**选项：**

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-d, --output-dir <DIR>` | 输出目录 | 当前目录 |
| `--password <P>` | 解密密码 | 无 |
| `--overwrite` | 覆盖现有文件 | 否 |

**示例：**

```bash
# 解压到当前目录
litepack unpack archive.zip

# 解压到指定目录
litepack unpack archive.zip -d ./output

# 带密码解压
litepack unpack encrypted.7z --password mypass

# 覆盖已存在文件
litepack unpack archive.zip --overwrite
```

### list - 列出内容

```bash
litepack list <archive>
```

**参数：**

| 参数 | 说明 | 必填 |
|------|------|:----:|
| `<archive>` | 压缩包路径 | 是 |

**示例：**

```bash
# 列出 ZIP 内容
litepack list archive.zip

# 列出 7z 内容
litepack list archive.7z
```

## 退出码

| 退出码 | 含义 |
|:------:|------|
| 0 | 操作成功 |
| 1 | 用户取消 (Ctrl+C) |
| 2 | 错误 |

## 进度显示

```
⠋ 压缩中... ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  60%  2.1/3.5 MB  00:12
```

- 旋转指示器 (spinner)
- 进度百分比
- 已完成/总字节数
- 预估剩余时间 (ETA)

## Ctrl+C 取消

按下 Ctrl+C 可随时取消正在进行的操作：
- 已创建的文件会被清理
- 返回退出码 1
- 显示取消提示

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `NO_COLOR` | 禁用彩色输出 | 未设置 |

## 错误处理

### 常见错误

| 错误信息 | 原因 | 解决方案 |
|----------|------|----------|
| 文件不存在 | 源文件路径错误 | 检查文件路径 |
| 不支持的格式 | 文件不是 ZIP/7z | 检查文件格式 |
| 密码错误 | 提供了错误的密码 | 确认密码 |
| 需要密码 | 加密文件未提供密码 | 添加 `--password` 参数 |
| 输出已存在 | 输出文件已存在 | 添加 `--overwrite` 参数 |

## 集成示例

### Shell 脚本

```bash
#!/bin/bash
# 批量解压目录下所有 ZIP 文件
for f in *.zip; do
    litepack unpack "$f" -d "./extracted/${f%.zip}"
done
```

### Windows 批处理

```batch
@echo off
REM 批量解压
for %%f in (*.zip) do (
    litepack unpack "%%f" -d ".\extracted\%%~nf"
)
```

### CI/CD 集成

```yaml
# GitHub Actions
- name: Extract archive
  run: |
    litepack unpack release.zip -d ./dist
    echo "Extracted successfully"
```

## 测试

```bash
# 运行 CLI 集成测试
cargo test -p litepack-cli

# 手动测试
cargo run -p litepack-cli -- pack test_files/ -o test.zip
cargo run -p litepack-cli -- list test.zip
cargo run -p litepack-cli -- unpack test.zip -d ./output
```

---

> 文档结束
