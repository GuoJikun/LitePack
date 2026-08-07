use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::path::Path;

use sevenz_rust2::encoder_options::{AesEncoderOptions, Lzma2Options};
use sevenz_rust2::{
    ArchiveEntry, ArchiveReader, ArchiveWriter, EncoderConfiguration, EncoderMethod, Error as SevenzError,
    Password,
};

use crate::entry::EntryInfo;
use crate::error::{Error, Result};
use crate::options::{CompressOptions, ExtractOptions, OverwriteMode};
use crate::progress::{CancelHandle, Phase, ProgressReport, ProgressSink};
use crate::walk::CompressEntry;
use crate::zip::safe_join;

/// Zip 炸弹安全上限（与 ZIP 一致）。
const MAX_ENTRIES: usize = 100_000;
const MAX_TOTAL_SIZE: u64 = 256 * 1024 * 1024 * 1024; // 256 GiB
const MAX_RATIO: u64 = 10_000;

pub fn list(archive: &Path) -> Result<Vec<EntryInfo>> {
    let file = File::open(archive)?;
    let reader = ArchiveReader::new(file, Password::empty()).map_err(|e| map_7z_error(e, false))?;
    let arch = reader.archive();
    if arch.files.len() > MAX_ENTRIES {
        return Err(Error::ZipBomb);
    }
    Ok(arch
        .files
        .iter()
        .map(|f| EntryInfo {
            path: f.name.clone(),
            size: f.size,
            is_dir: f.is_directory,
            compressed_size: (f.compressed_size > 0).then_some(f.compressed_size),
            method: None,
        })
        .collect())
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
    let mut writer = ArchiveWriter::new(file).map_err(|e| map_7z_error(e, false))?;

    let mut methods: Vec<EncoderConfiguration> = Vec::new();
    if let Some(pw) = &opts.password {
        methods.push(AesEncoderOptions::new(Password::from(pw.as_str())).into());
    }
    if opts.level == 0 {
        methods.push(EncoderConfiguration::new(EncoderMethod::COPY));
    } else {
        methods.push(
            EncoderConfiguration::new(EncoderMethod::LZMA2)
                .with_options(Lzma2Options::from_level(opts.level).into()),
        );
    }
    writer.set_content_methods(methods);

    let mut done: u64 = 0;
    for e in entries {
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        sink.update(&ProgressReport {
            phase: Phase::Compressing,
            current_file: e.src.clone(),
            done_bytes: done,
            total_bytes: total,
        });

        if e.name.ends_with('/') {
            writer
                .push_archive_entry::<io::Empty>(ArchiveEntry::new_directory(&e.name), None)
                .map_err(|err| map_7z_error(err, false))?;
        } else {
            let entry = ArchiveEntry::from_path(&e.src, e.name.clone());
            let src = File::open(&e.src)?;
            writer
                .push_archive_entry(entry, Some(src))
                .map_err(|err| map_7z_error(err, false))?;
            done += e.src.metadata().map(|m| m.len()).unwrap_or(0);
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
    let has_password = opts.password.is_some();
    let password = match &opts.password {
        Some(p) => Password::from(p.as_str()),
        None => Password::empty(),
    };
    let file = File::open(archive)?;
    let mut reader = ArchiveReader::new(file, password).map_err(|e| map_7z_error(e, has_password))?;

    let arch = reader.archive();
    if arch.files.len() > MAX_ENTRIES {
        return Err(Error::ZipBomb);
    }
    let total: u64 = arch.files.iter().map(|f| f.size).sum();
    if total > MAX_TOTAL_SIZE {
        return Err(Error::ZipBomb);
    }
    let compressed = std::fs::metadata(archive).map(|m| m.len()).unwrap_or(0);
    if compressed > 0 && total / compressed > MAX_RATIO {
        return Err(Error::ZipBomb);
    }

    let mut done: u64 = 0;
    let mut failed: Option<Error> = None;
    let result = reader.for_each_entries(|entry, entry_reader| {
        if cancel.is_cancelled() {
            failed = Some(Error::Cancelled);
            return Err(fail());
        }
        let out_path = match safe_join(out_dir, entry.name()) {
            Ok(p) => p,
            Err(e) => {
                failed = Some(e);
                return Err(fail());
            }
        };
        if entry.is_directory() {
            if let Err(e) = std::fs::create_dir_all(&out_path) {
                failed = Some(Error::Io(e));
                return Err(fail());
            }
            return Ok(true);
        }
        if out_path.exists() {
            match opts.overwrite {
                OverwriteMode::Skip => {
                    // 仍需读完数据以推进 solid 块解码。
                    let _ = io::copy(entry_reader, &mut io::sink());
                    return Ok(true);
                }
                OverwriteMode::Overwrite => {}
                OverwriteMode::Ask => {
                    failed = Some(Error::Io(io::Error::other(
                        "OverwriteMode::Ask 需要调用方在回调中处理",
                    )));
                    return Err(fail());
                }
            }
        }
        if let Some(parent) = out_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                failed = Some(Error::Io(e));
                return Err(fail());
            }
        }
        sink.update(&ProgressReport {
            phase: Phase::Extracting,
            current_file: out_path.clone(),
            done_bytes: done,
            total_bytes: total,
        });
        match File::create(&out_path) {
            Ok(mut f) => {
                if let Err(e) = io::copy(entry_reader, &mut f) {
                    failed = Some(Error::Io(e));
                    return Err(fail());
                }
            }
            Err(e) => {
                failed = Some(Error::Io(e));
                return Err(fail());
            }
        }
        done += entry.size();
        Ok(true)
    });
    if let Some(e) = failed {
        return Err(e);
    }
    result.map_err(|e| map_7z_error(e, has_password))?;

    sink.update(&ProgressReport {
        phase: Phase::Extracting,
        current_file: out_dir.to_path_buf(),
        done_bytes: total,
        total_bytes: total,
    });
    Ok(())
}

/// 构造一个用于中断 sevenz 迭代的哨兵错误。
fn fail() -> SevenzError {
    SevenzError::Other(Cow::Borrowed("litepack internal error"))
}

fn map_7z_error(e: SevenzError, has_password: bool) -> Error {
    use SevenzError as E;
    match e {
        E::BadSignature(_) => Error::UnsupportedFormat("不是有效的 7z 文件".into()),
        E::UnsupportedVersion { .. } => Error::UnsupportedFormat(format!("不支持的 7z 版本: {e}")),
        E::PasswordRequired | E::MaybeBadPassword(_) => {
            if has_password {
                Error::BadPassword
            } else {
                Error::Encrypted
            }
        }
        E::ChecksumVerificationFailed | E::NextHeaderCrcMismatch => {
            Error::Io(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        }
        E::Io(inner, msg) => Error::Io(io::Error::new(inner.kind(), format!("{msg}: {inner}"))),
        E::FileOpen(inner, msg) => Error::Io(io::Error::new(inner.kind(), msg)),
        other => Error::Io(io::Error::other(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct NoopSink;

    impl ProgressSink for NoopSink {
        fn update(&self, _r: &ProgressReport) {}
    }

    fn temp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("litepack-7z-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn roundtrip_dir() {
        let root = temp("rt-dir");
        let src = root.join("data");
        std::fs::create_dir_all(src.join("empty_dir")).unwrap();
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), b"hello world").unwrap();
        std::fs::write(src.join("sub/b.txt"), vec![b'x'; 10_000]).unwrap();

        let archive = root.join("out.7z");
        let entries = crate::walk::collect_entries(std::slice::from_ref(&src), &CompressOptions::default()).unwrap();
        compress(&entries, &archive, &CompressOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();

        let listed = list(&archive).unwrap();
        let names: Vec<&str> = listed.iter().map(|e| e.path.as_str()).collect();
        assert!(names.contains(&"data/a.txt"));
        assert!(names.contains(&"data/sub/b.txt"));
        assert!(names.iter().any(|n| ["data/empty_dir", "data/empty_dir/", "data/empty_dir\\"].contains(n)));

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
        let archive = root.join("enc.7z");
        let opts = CompressOptions {
            password: Some("pass123".into()),
            ..CompressOptions::default()
        };
        let entries = crate::walk::collect_entries(std::slice::from_ref(&f), &opts).unwrap();
        compress(&entries, &archive, &opts, &NoopSink, &CancelHandle::new()).unwrap();

        let ok_opts = ExtractOptions {
            password: Some("pass123".into()),
            ..ExtractOptions::default()
        };
        let out = root.join("ok");
        decompress(&archive, &out, &ok_opts, &NoopSink, &CancelHandle::new()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("secret.txt")).unwrap(), "top secret");

        let wrong_opts = ExtractOptions {
            password: Some("nope".into()),
            ..ExtractOptions::default()
        };
        let r = decompress(&archive, &root.join("wrong"), &wrong_opts, &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::BadPassword)), "{r:?}");

        let r = decompress(
            &archive,
            &root.join("nopw"),
            &ExtractOptions::default(),
            &NoopSink,
            &CancelHandle::new(),
        );
        assert!(matches!(r, Err(Error::Encrypted)), "{r:?}");
    }

    #[test]
    fn cancel_mid_operation() {
        let root = temp("cancel");
        let big = root.join("big.bin");
        std::fs::write(&big, vec![0u8; 64 * 1024]).unwrap();
        let archive = root.join("big.7z");
        let entries = crate::walk::collect_entries(std::slice::from_ref(&big), &CompressOptions::default()).unwrap();
        compress(&entries, &archive, &CompressOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();

        let cancel = CancelHandle::new();
        cancel.cancel();
        let out = root.join("out");
        let r = decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &cancel);
        assert!(matches!(r, Err(Error::Cancelled)), "{r:?}");
    }

    #[test]
    fn reject_traversal_entry() {
        let root = temp("traversal");
        let archive = root.join("evil.7z");
        let f = File::create(&archive).unwrap();
        let mut w = ArchiveWriter::new(f).unwrap();
        w.set_content_methods(vec![EncoderConfiguration::new(EncoderMethod::COPY)]);
        let entry = ArchiveEntry::new_file("../evil.txt");
        w.push_archive_entry::<io::Empty>(entry, None).unwrap();
        w.finish().unwrap();

        let out = root.join("out");
        let r = decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::PathTraversal(_))), "{r:?}");
    }

    #[test]
    fn reject_too_many_entries() {
        let root = temp("many");
        let archive = root.join("many.7z");
        let f = File::create(&archive).unwrap();
        let mut w = ArchiveWriter::new(f).unwrap();
        w.set_content_methods(vec![EncoderConfiguration::new(EncoderMethod::COPY)]);
        for i in 0..=100_000 {
            w.push_archive_entry::<io::Empty>(ArchiveEntry::new_file(&format!("f{i}")), None)
                .unwrap();
        }
        w.finish().unwrap();

        let out = root.join("out");
        let r = decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new());
        assert!(matches!(r, Err(Error::ZipBomb)), "{r:?}");
    }

    #[test]
    fn interop_with_system_7z() {
        let has_7z = std::process::Command::new("7z").arg("i").output().map(|o| o.status.success()).unwrap_or(false);
        if !has_7z {
            eprintln!("系统 7z 不可用，跳过互操作测试");
            return;
        }
        let root = temp("interop");
        let archive = root.join("sys.7z");
        let src = root.join("payload");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), "你好 world").unwrap();
        std::fs::write(src.join("sub/b.bin"), vec![0xAB; 512]).unwrap();

        let status = std::process::Command::new("7z")
            .arg("a")
            .arg(&archive)
            .arg(&src)
            .status()
            .unwrap();
        assert!(status.success());

        let listed = list(&archive).unwrap();
        let names: Vec<&str> = listed.iter().map(|e| e.path.as_str()).collect();
        assert!(names.iter().any(|n| n.ends_with("a.txt")));

        let out = root.join("out");
        decompress(&archive, &out, &ExtractOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();
        let found = out.join("payload/a.txt");
        assert!(found.exists(), "提取路径不存在: {}", found.display());

        // 反向：我们生成的 7z 用系统 7z 解压。
        let ours = root.join("ours.7z");
        let entries = crate::walk::collect_entries(std::slice::from_ref(&src), &CompressOptions::default()).unwrap();
        compress(&entries, &ours, &CompressOptions::default(), &NoopSink, &CancelHandle::new()).unwrap();
        let verify = root.join("verify");
        let status = std::process::Command::new("7z")
            .arg("x")
            .arg(&ours)
            .arg(format!("-o{}", verify.display()))
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(
            std::fs::read_to_string(verify.join("payload/a.txt")).unwrap(),
            "你好 world"
        );
    }
}
