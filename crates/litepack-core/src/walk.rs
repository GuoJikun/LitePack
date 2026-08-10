use std::path::Path;

use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::options::CompressOptions;

/// 单个待压缩源及其归档内名称。
#[derive(Clone, Debug)]
pub struct CompressEntry {
    /// 源文件/目录在磁盘上的绝对路径。
    pub src: std::path::PathBuf,
    /// 归档内相对名称（`/` 分隔；目录以 `/` 结尾）。
    pub name: String,
}

/// 收集待压缩条目：目录递归产生名称（含顶层目录名），文件直接加入。
/// 隐藏文件/目录在 `skip_hidden` 时被排除。
pub fn collect_entries(
    inputs: &[std::path::PathBuf],
    opts: &CompressOptions,
) -> Result<Vec<CompressEntry>> {
    let mut out = Vec::new();
    for input in inputs {
        if opts.skip_hidden && is_hidden(input) {
            continue;
        }
        if input.is_dir() {
            let top = input
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    Error::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "无效目录名",
                    ))
                })?
                .to_string();
            out.push(CompressEntry {
                src: input.clone(),
                name: format!("{top}/"),
            });
            for entry in WalkDir::new(input)
                .follow_links(false)
                .into_iter()
                .filter_entry(|e| !(opts.skip_hidden && is_hidden(e.path())))
            {
                let entry = entry.map_err(std::io::Error::from)?;
                let rel = entry.path().strip_prefix(input).map_err(|_| {
                    Error::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "路径解析失败",
                    ))
                })?;
                if rel.as_os_str().is_empty() {
                    continue;
                }
                let rel_str = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                let name = if entry.file_type().is_dir() {
                    format!("{top}/{rel_str}/")
                } else {
                    format!("{top}/{rel_str}")
                };
                out.push(CompressEntry {
                    src: entry.into_path(),
                    name,
                });
            }
        } else {
            let name = input
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push(CompressEntry {
                src: input.clone(),
                name,
            });
        }
    }
    Ok(out)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_files_and_dirs() {
        let dir = std::env::temp_dir().join(format!("litepack-walk-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.txt"), b"a").unwrap();
        std::fs::write(dir.join("sub/b.txt"), b"b").unwrap();
        std::fs::write(dir.join(".hidden"), b"h").unwrap();

        let entries =
            collect_entries(std::slice::from_ref(&dir), &CompressOptions::default()).unwrap();
        let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
        let dirname = dir.file_name().unwrap().to_string_lossy().into_owned();
        assert!(names.contains(&format!("{dirname}/")));
        assert!(names.contains(&format!("{dirname}/a.txt")));
        assert!(names.contains(&format!("{dirname}/sub/b.txt")));
        assert!(!names.iter().any(|n| n.contains(".hidden")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn single_file_keeps_basename() {
        let dir = std::env::temp_dir().join(format!("litepack-walk-file-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("x.txt");
        std::fs::write(&f, b"x").unwrap();
        let entries =
            collect_entries(std::slice::from_ref(&f), &CompressOptions::default()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "x.txt");
    }
}
