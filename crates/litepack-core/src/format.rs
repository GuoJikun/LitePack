use std::path::Path;

use crate::error::{Error, Result};

/// 支持的归档格式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    SevenZ,
}

/// ZIP 魔数：本地文件头 / 中央目录 / EOCD 均以 "PK" 开头。
const ZIP_MAGIC: &[u8] = &[0x50, 0x4B];
/// 7z 魔数：`37 7A BC AF 27 1C`。
const SEVENZ_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

impl ArchiveFormat {
    /// 由扩展名推断格式（不区分大小写）。
    pub fn from_extension(path: &Path) -> Result<ArchiveFormat> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("zip") => Ok(ArchiveFormat::Zip),
            Some("7z") => Ok(ArchiveFormat::SevenZ),
            other => Err(Error::UnsupportedFormat(other.unwrap_or("(no extension)").to_string())),
        }
    }
}

/// 读取文件头魔数识别格式；魔数无法识别时回退到扩展名。
pub fn detect_format(path: &Path) -> Result<ArchiveFormat> {
    let mut head = [0u8; SEVENZ_MAGIC.len()];
    let mut file = std::fs::File::open(path)
        .map_err(|e| Error::Io(std::io::Error::new(e.kind(), format!("open {}: {e}", path.display()))))?;
    use std::io::Read;
    let n = file.read(&mut head)?;

    if head.starts_with(ZIP_MAGIC) {
        return Ok(ArchiveFormat::Zip);
    }
    if n == SEVENZ_MAGIC.len() && head == *SEVENZ_MAGIC {
        return Ok(ArchiveFormat::SevenZ);
    }
    ArchiveFormat::from_extension(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("litepack-format-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        p
    }

    fn write_bytes(dir: &std::path::Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let p = dir.join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(bytes).unwrap();
        p
    }

    #[test]
    fn detects_zip_by_magic() {
        let dir = temp_dir();
        let p = write_bytes(&dir, "x.zip", &[0x50, 0x4B, 0x03, 0x04, 0x00, 0x00]);
        assert_eq!(detect_format(&p).unwrap(), ArchiveFormat::Zip);
    }

    #[test]
    fn detects_empty_zip_by_magic() {
        let dir = temp_dir();
        let p = write_bytes(&dir, "empty.zip", &[0x50, 0x4B, 0x05, 0x06, 0x00, 0x00]);
        assert_eq!(detect_format(&p).unwrap(), ArchiveFormat::Zip);
    }

    #[test]
    fn detects_sevenz_by_magic() {
        let dir = temp_dir();
        let p = write_bytes(&dir, "x.7z", &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]);
        assert_eq!(detect_format(&p).unwrap(), ArchiveFormat::SevenZ);
    }

    #[test]
    fn falls_back_to_extension() {
        let dir = temp_dir();
        let p = write_bytes(&dir, "weird.zip", b"not really a zip");
        assert_eq!(detect_format(&p).unwrap(), ArchiveFormat::Zip);
        let q = write_bytes(&dir, "weird.7z", b"not really a 7z");
        assert_eq!(detect_format(&q).unwrap(), ArchiveFormat::SevenZ);
    }

    #[test]
    fn unknown_extension_rejected() {
        let dir = temp_dir();
        let p = write_bytes(&dir, "x.rar", b"data");
        assert!(matches!(detect_format(&p), Err(Error::UnsupportedFormat(_))));
    }
}
