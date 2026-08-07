use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{AesMode, CompressionMethod, ZipArchive, ZipWriter};

use crate::entry::EntryInfo;
use crate::error::{Error, Result};
use crate::options::{CompressOptions, ExtractOptions, OverwriteMode};
use crate::progress::{CancelHandle, Phase, ProgressReport, ProgressSink};
use crate::walk::CompressEntry;

/// Zip 炸弹安全上限。
const MAX_ENTRIES: usize = 100_000;
const MAX_TOTAL_SIZE: u64 = 256 * 1024 * 1024 * 1024; // 256 GiB
const MAX_RATIO: u64 = 10_000;

pub fn list(archive: &Path) -> Result<Vec<EntryInfo>> {
    let file = File::open(archive)?;
    let mut z = ZipArchive::new(file)?;
    if z.len() > MAX_ENTRIES {
        return Err(Error::ZipBomb);
    }
    let mut out = Vec::with_capacity(z.len());
    for i in 0..z.len() {
        let e = z.by_index_raw(i)?;
        out.push(EntryInfo {
            path: e.name().to_string(),
            size: e.size(),
            is_dir: e.is_dir(),
            compressed_size: Some(e.compressed_size()),
            method: Some(e.compression().to_string()),
        });
    }
    Ok(out)
}

pub fn compress(
    entries: &[CompressEntry],
    out: &Path,
    opts: &CompressOptions,
    sink: &dyn ProgressSink,
    cancel: &CancelHandle,
) -> Result<()> {
    if out.exists() && !opts.overwrite {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("输出已存在: {}", out.display()),
        )));
    }
    let total: u64 = entries
        .iter()
        .filter(|e| !e.name.ends_with('/'))
        .filter_map(|e| e.src.metadata().ok())
        .map(|m| m.len())
        .sum();

    let file = File::create(out)?;
    let mut writer = ZipWriter::new(file);
    let mut done: u64 = 0;

    for e in entries {
        if cancel.is_cancelled() {
            let _ = writer.finish();
            return Err(Error::Cancelled);
        }
        sink.update(&ProgressReport {
            phase: Phase::Compressing,
            current_file: e.src.clone(),
            done_bytes: done,
            total_bytes: total,
        });

        let mut file_opts = SimpleFileOptions::default()
            .compression_method(if opts.level == 0 {
                CompressionMethod::Stored
            } else {
                CompressionMethod::Deflated
            })
            .compression_level(Some(opts.level as i64))
            .large_file(true);
        if let Some(pw) = &opts.password {
            file_opts = file_opts.with_aes_encryption(AesMode::Aes256, pw.as_str());
        }

        if e.name.ends_with('/') {
            writer.add_directory(&e.name, file_opts)?;
        } else {
            writer.start_file(&e.name, file_opts)?;
            let mut f = File::open(&e.src)?;
            let n = io::copy(&mut f, &mut writer)?;
            done += n;
        }
    }

    sink.update(&ProgressReport {
        phase: Phase::Compressing,
        current_file: out.to_path_buf(),
        done_bytes: total,
        total_bytes: total,
    });
    writer.finish()?;
    Ok(())
}

pub fn decompress(
    archive: &Path,
    out_dir: &Path,
    opts: &ExtractOptions,
    sink: &dyn ProgressSink,
    cancel: &CancelHandle,
) -> Result<()> {
    let file = File::open(archive)?;
    let mut z = ZipArchive::new(file)?;
    if z.len() > MAX_ENTRIES {
        return Err(Error::ZipBomb);
    }
    check_bomb(&mut z, archive)?;

    let total: u64 = (0..z.len())
        .map(|i| z.by_index_raw(i).map(|e| e.size()).unwrap_or(0))
        .sum();
    let mut done: u64 = 0;

    for i in 0..z.len() {
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }

        let plain = z.by_index_raw(i)?;
        let encrypted = plain.encrypted();
        let name = plain.name().to_string();
        let is_dir = plain.is_dir();
        drop(plain);

        if encrypted && opts.password.is_none() {
            return Err(Error::Encrypted);
        }

        let out_path = safe_join(out_dir, &name)?;
        if is_dir {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if out_path.exists() {
            match opts.overwrite {
                OverwriteMode::Skip => continue,
                OverwriteMode::Overwrite => {}
                OverwriteMode::Ask => {
                    return Err(Error::Io(io::Error::other(
                        "OverwriteMode::Ask 需要调用方在回调中处理",
                    )));
                }
            }
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        sink.update(&ProgressReport {
            phase: Phase::Extracting,
            current_file: out_path.clone(),
            done_bytes: done,
            total_bytes: total,
        });

        let mut entry = if encrypted {
            let pw = opts.password.as_deref().expect("checked above");
            z.by_index_decrypt(i, pw.as_bytes())?
        } else {
            z.by_index(i)?
        };
        let mut out_f = File::create(&out_path)?;
        let n = io::copy(&mut entry, &mut out_f).map_err(|e| map_copy_error(&e, encrypted))?;
        done += n;
    }

    sink.update(&ProgressReport {
        phase: Phase::Extracting,
        current_file: out_dir.to_path_buf(),
        done_bytes: total,
        total_bytes: total,
    });
    Ok(())
}

fn check_bomb(z: &mut ZipArchive<File>, archive: &Path) -> Result<()> {
    let compressed = std::fs::metadata(archive).map(|m| m.len()).unwrap_or(0);
    let mut uncompressed: u64 = 0;
    for i in 0..z.len() {
        let e = z.by_index_raw(i)?;
        uncompressed = uncompressed.saturating_add(e.size());
        if uncompressed > MAX_TOTAL_SIZE {
            return Err(Error::ZipBomb);
        }
    }
    if compressed > 0 && uncompressed / compressed > MAX_RATIO {
        return Err(Error::ZipBomb);
    }
    Ok(())
}

/// 安全拼接输出路径：拒绝绝对路径、盘符前缀与 `..` 组件。
pub(crate) fn safe_join(out_dir: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty() {
        return Err(Error::PathTraversal("空条目名".into()));
    }
    let normalized = name.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(Error::PathTraversal(name.into()));
    }
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => return Err(Error::PathTraversal(name.into())),
            p if p.contains(':') => return Err(Error::PathTraversal(name.into())),
            _ => {}
        }
    }
    Ok(out_dir.join(name))
}

fn map_copy_error(e: &io::Error, encrypted: bool) -> Error {
    if encrypted && e.kind() == io::ErrorKind::InvalidInput {
        Error::BadPassword
    } else {
        Error::Io(io::Error::new(e.kind(), e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct NoopSink;

    impl ProgressSink for NoopSink {
        fn update(&self, _r: &ProgressReport) {}
    }

    fn temp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("litepack-zip-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn make_fixture(root: &Path) {
        std::fs::create_dir_all(root.join("empty_dir")).unwrap();
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("a.txt"), b"hello world").unwrap();
        std::fs::write(root.join("sub/b.txt"), vec![b'x'; 10_000]).unwrap();
    }

    #[test]
    fn roundtrip_dir() {
        let root = temp("rt-dir");
        let src = root.join("data");
        std::fs::create_dir_all(&src).unwrap();
        make_fixture(&src);

        let archive = root.join("out.zip");
        let entries = crate::walk::collect_entries(std::slice::from_ref(&src), &CompressOptions::default()).unwrap();
        compress(&entries, &archive, &CompressOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();

        let listed = list(&archive).unwrap();
        let names: Vec<&str> = listed.iter().map(|e| e.path.as_str()).collect();
        assert!(names.contains(&"data/"));
        assert!(names.contains(&"data/a.txt"));
        assert!(names.contains(&"data/sub/b.txt"));
        assert!(names.contains(&"data/empty_dir/"));

        let out = root.join("extracted");
        decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("data/a.txt")).unwrap(), "hello world");
        assert_eq!(std::fs::read(out.join("data/sub/b.txt")).unwrap().len(), 10_000);
        assert!(out.join("data/empty_dir").is_dir());
    }

    #[test]
    fn roundtrip_password() {
        let root = temp("rt-pw");
        let f = root.join("secret.txt");
        std::fs::write(&f, b"top secret").unwrap();
        let archive = root.join("enc.zip");
        let opts = CompressOptions {
            password: Some("pass123".into()),
            ..CompressOptions::default()
        };
        let entries = crate::walk::collect_entries(std::slice::from_ref(&f), &opts).unwrap();
        compress(&entries, &archive, &opts, &NoopSink, &CancelHandle::new()).unwrap();

        let out_opts = ExtractOptions {
            password: Some("pass123".into()),
            ..ExtractOptions::default()
        };
        let out = root.join("ok");
        decompress(&archive, &out, &out_opts, &NoopSink, &CancelHandle::new()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("secret.txt")).unwrap(), "top secret");

        let out2 = root.join("wrong");
        let wrong_opts = ExtractOptions {
            password: Some("nope".into()),
            ..ExtractOptions::default()
        };
        let r = decompress(&archive, &out2, &wrong_opts, &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::BadPassword)));

        let out3 = root.join("nopw");
        let r = decompress(&archive, &out3, &ExtractOptions::default(), &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::Encrypted)));
    }

    #[test]
    fn reject_traversal() {
        let root = temp("traversal");
        let archive = root.join("evil.zip");
        {
            let f = File::create(&archive).unwrap();
            let mut w = ZipWriter::new(f);
            let opts = SimpleFileOptions::default();
            w.start_file("../evil.txt", opts).unwrap();
            w.write_all(b"pwn").unwrap();
            w.finish().unwrap();
        }
        let out = root.join("out");
        let r = decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::PathTraversal(_))));
    }

    #[test]
    fn safe_join_rejects_absolute_and_drive() {
        for evil in [
            "/etc/passwd",
            "C:\\windows\\system32\\evil.dll",
            "C:/windows/evil.exe",
            "a/../b",
            "..\\..\\x",
            "..",
            "",
        ] {
            let r = safe_join(Path::new("out"), evil);
            assert!(r.is_err(), "{evil:?} 应当被拒绝，实际 {r:?}");
        }
        // 正常路径必须放行。
        assert!(safe_join(Path::new("out"), "a/b/c.txt").is_ok());
        assert!(safe_join(Path::new("out"), "中文/名.txt").is_ok());
    }

    #[test]
    fn cancel_mid_operation() {
        let root = temp("cancel");
        let big = root.join("big.bin");
        std::fs::write(&big, vec![0u8; 64 * 1024]).unwrap();
        let archive = root.join("big.zip");
        let entries = crate::walk::collect_entries(std::slice::from_ref(&big), &CompressOptions::default()).unwrap();
        compress(&entries, &archive, &CompressOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();

        let cancel = CancelHandle::new();
        cancel.cancel();
        let out = root.join("out");
        let r = decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &cancel);
        assert!(matches!(r, Err(Error::Cancelled)));
    }

    #[test]
    fn reject_zip_bomb() {
        let root = temp("bomb");
        let archive = root.join("bomb.zip");
        craft_zip_bomb(&archive, 100_000_000);
        // 声称 100MiB 解压后内容，实际仅 4 字节存储数据，比例远超 10000。
        let out = root.join("out");
        let r = decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::ZipBomb)), "{r:?}");
    }

    /// 手工构造单条目 Stored ZIP，中央目录声称 `fake_size` 解压大小。
    fn craft_zip_bomb(path: &Path, fake_size: u32) {
        use std::io::Write;
        let name = b"bomb.bin";
        let data = b"\x00\x01\x02\x03";
        let crc = 0x1234_5678u32;

        let mut local = Vec::new();
        local.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // 本地头签名
        local.extend_from_slice(&20u16.to_le_bytes()); // version
        local.extend_from_slice(&0u16.to_le_bytes()); // flags
        local.extend_from_slice(&0u16.to_le_bytes()); // method = Stored
        local.extend_from_slice(&0u16.to_le_bytes()); // time
        local.extend_from_slice(&0x21u16.to_le_bytes()); // date
        local.extend_from_slice(&crc.to_le_bytes());
        local.extend_from_slice(&(data.len() as u32).to_le_bytes()); // csize
        local.extend_from_slice(&fake_size.to_le_bytes()); // usize
        local.extend_from_slice(&(name.len() as u16).to_le_bytes());
        local.extend_from_slice(&0u16.to_le_bytes()); // extra len
        local.extend_from_slice(name);
        local.extend_from_slice(data);

        let local_offset = 0u32;
        let mut central = Vec::new();
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes()); // 中央目录签名
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0u16.to_le_bytes()); // flags
        central.extend_from_slice(&0u16.to_le_bytes()); // method
        central.extend_from_slice(&0u16.to_le_bytes()); // time
        central.extend_from_slice(&0x21u16.to_le_bytes()); // date
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes()); // csize
        central.extend_from_slice(&fake_size.to_le_bytes()); // usize
        central.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra len
        central.extend_from_slice(&0u16.to_le_bytes()); // comment len
        central.extend_from_slice(&0u16.to_le_bytes()); // disk start
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&local_offset.to_le_bytes());
        central.extend_from_slice(name);

        let mut eocd = Vec::new();
        eocd.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // EOCD 签名
        eocd.extend_from_slice(&0u16.to_le_bytes()); // disk num
        eocd.extend_from_slice(&0u16.to_le_bytes()); // cd disk
        eocd.extend_from_slice(&1u16.to_le_bytes()); // entries this disk
        eocd.extend_from_slice(&1u16.to_le_bytes()); // total entries
        eocd.extend_from_slice(&(central.len() as u32).to_le_bytes()); // cd size
        eocd.extend_from_slice(&(local.len() as u32).to_le_bytes()); // cd offset
        eocd.extend_from_slice(&0u16.to_le_bytes()); // comment len

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&local);
        bytes.extend_from_slice(&central);
        bytes.extend_from_slice(&eocd);
        let mut f = File::create(path).unwrap();
        f.write_all(&bytes).unwrap();
    }

    #[test]
    fn interop_with_python_zipfile() {
        let py = std::process::Command::new("python")
            .arg("--version")
            .output();
        let py = match py {
            Ok(o) if o.status.success() => "python",
            _ => {
                if std::process::Command::new("py").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
                    "py"
                } else {
                    eprintln!("python 不可用，跳过互操作测试");
                    return;
                }
            }
        };
        let root = temp("interop");
        let archive = root.join("py.zip");
        let script = format!(
            r#"import zipfile, os
with zipfile.ZipFile({archive:?}, "w", zipfile.ZIP_DEFLATED) as z:
    z.writestr("hello.txt", "你好 world")
    z.writestr("dir/中文名.bin", bytes(range(256)))"#
        );
        let status = std::process::Command::new(py)
            .arg("-c")
            .arg(&script)
            .status()
            .unwrap();
        assert!(status.success());

        let listed = list(&archive).unwrap();
        let names: Vec<&str> = listed.iter().map(|e| e.path.as_str()).collect();
        assert!(names.contains(&"hello.txt"));
        assert!(names.contains(&"dir/中文名.bin"));

        let out = root.join("out");
        decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("hello.txt")).unwrap(), "你好 world");
        assert_eq!(std::fs::read(out.join("dir/中文名.bin")).unwrap().len(), 256);

        let back = root.join("back.zip");
        let entries = crate::walk::collect_entries(std::slice::from_ref(&out), &CompressOptions::default()).unwrap();
        compress(&entries, &back, &CompressOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();
        let verify = format!(
            r#"import zipfile, sys
with zipfile.ZipFile({back:?}) as z:
    assert "hello.txt" in z.namelist(), z.namelist()
    assert z.read("hello.txt") == "你好 world".encode()
print("OK")"#
        );
        let out = std::process::Command::new(py)
            .arg("-c")
            .arg(&verify)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    }
}
