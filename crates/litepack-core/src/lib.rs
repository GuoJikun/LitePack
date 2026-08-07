//! LitePack 纯 Rust 压缩/解压核心库，仅支持 ZIP 与 7z。

pub mod entry;
pub mod error;
pub mod format;
pub mod options;
pub mod progress;
pub mod sevenz;
pub mod walk;
pub mod zip;

pub use entry::EntryInfo;
pub use error::{Error, Result};
pub use format::ArchiveFormat;
pub use options::{CompressOptions, ExtractOptions, OverwriteMode};
pub use progress::{CancelHandle, Phase, ProgressReport, ProgressSink};
pub use walk::{collect_entries, CompressEntry};

use std::path::Path;

/// 识别归档格式：优先魔数，扩展名兜底。
pub fn detect_format(path: &Path) -> Result<ArchiveFormat> {
    format::detect_format(path)
}

/// 列出归档内全部条目。
pub fn list(archive: &Path) -> Result<Vec<EntryInfo>> {
    match detect_format(archive)? {
        ArchiveFormat::Zip => zip::list(archive),
        ArchiveFormat::SevenZ => sevenz::list(archive),
    }
}

/// 压缩条目列表为归档文件。
pub fn compress(
    entries: &[CompressEntry],
    out: &Path,
    opts: &CompressOptions,
    sink: &dyn ProgressSink,
    cancel: &CancelHandle,
) -> Result<()> {
    match ArchiveFormat::from_extension(out)? {
        ArchiveFormat::Zip => zip::compress(entries, out, opts, sink, cancel),
        ArchiveFormat::SevenZ => sevenz::compress(entries, out, opts, sink, cancel),
    }
}

/// 解压归档到目标目录。
pub fn decompress(
    archive: &Path,
    out_dir: &Path,
    opts: &ExtractOptions,
    sink: &dyn ProgressSink,
    cancel: &CancelHandle,
) -> Result<()> {
    match detect_format(archive)? {
        ArchiveFormat::Zip => zip::decompress(archive, out_dir, opts, sink, cancel),
        ArchiveFormat::SevenZ => sevenz::decompress(archive, out_dir, opts, sink, cancel),
    }
}
