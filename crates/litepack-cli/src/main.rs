use std::fmt;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use litepack_core::{
    CancelHandle, CompressOptions, Error as CoreError, ExtractOptions, Phase, ProgressReport,
    ProgressSink, OverwriteMode,
};

#[derive(Parser)]
#[command(name = "litepack", version, about = "LitePack：纯 Rust 压缩/解压工具（ZIP / 7z）")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 压缩文件/目录到归档
    Pack {
        /// 源文件或目录（可指定多个）
        #[arg(required = true)]
        src: Vec<PathBuf>,
        /// 输出归档路径（.zip 或 .7z）
        #[arg(short = 'o', long)]
        output: PathBuf,
        /// 压缩级别 0-9（0 为不压缩）
        #[arg(short, long, default_value_t = 6)]
        level: u32,
        /// 加密密码
        #[arg(short, long)]
        password: Option<String>,
        /// 输出文件已存在时覆盖
        #[arg(long)]
        overwrite: bool,
        /// 跳过隐藏文件/目录
        #[arg(long)]
        skip_hidden: bool,
        /// 强制指定格式：zip 或 7z
        #[arg(long, value_parser = ["zip", "7z"])]
        format: Option<String>,
    },
    /// 解压归档到目录
    Unpack {
        /// 归档文件
        archive: PathBuf,
        /// 输出目录（默认当前目录）
        #[arg(short = 'd', long, default_value = ".")]
        output_dir: PathBuf,
        /// 加密密码
        #[arg(short, long)]
        password: Option<String>,
        /// 覆盖已存在文件
        #[arg(long)]
        overwrite: bool,
    },
    /// 列出归档内容
    List {
        /// 归档文件
        archive: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    match cli.command {
        Command::Pack { src, output, level, password, overwrite, skip_hidden, format } => {
            if let Some(fmt) = format {
                let ext = output
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_ascii_lowercase());
                if ext.as_deref() != Some(fmt.as_str()) {
                    eprintln!("错误: --format {fmt} 与输出扩展名 {} 不一致", ext.as_deref().unwrap_or("(无)"));
                    return 2;
                }
            }
            if level > 9 {
                eprintln!("错误: 压缩级别必须在 0-9 之间");
                return 2;
            }
            let opts = CompressOptions {
                level,
                password,
                overwrite,
                skip_hidden,
            };
            run_with_cancel(|cancel| {
                let entries = litepack_core::collect_entries(&src, &opts)?;
                let total: u64 = entries
                    .iter()
                    .filter(|e| !e.name.ends_with('/'))
                    .filter_map(|e| e.src.metadata().ok())
                    .map(|m| m.len())
                    .sum();
                let sink = CliProgress::new(total);
                litepack_core::compress(&entries, &output, &opts, &sink, cancel)
                    .map_err(anyhow::Error::from)
            })
        }
        Command::Unpack { archive, output_dir, password, overwrite } => {
            let opts = ExtractOptions {
                password,
                overwrite: if overwrite { OverwriteMode::Overwrite } else { OverwriteMode::Skip },
            };
            run_with_cancel(|cancel| {
                let sink = CliProgress::new(0);
                litepack_core::decompress(&archive, &output_dir, &opts, &sink, cancel)
                    .map_err(anyhow::Error::from)
            })
        }
        Command::List { archive } => {
            match litepack_core::list(&archive) {
                Ok(entries) => {
                    if entries.is_empty() {
                        println!("(空归档)");
                    }
                    for e in &entries {
                        let size = human_size(e.size);
                        let method = e.method.as_deref().unwrap_or("-");
                        let name = e.path.trim_end_matches(['/', '\\']);
                        if e.is_dir {
                            println!("{:<8} {:>10}  {}/", "目录", size, name);
                        } else {
                            println!("{:<8} {:>10}  {}", method, size, name);
                        }
                    }
                    0
                }
                Err(e) => {
                    eprintln!("错误: {e}");
                    exit_code(&e)
                }
            }
        }
    }
}

/// 执行操作并处理 Ctrl+C 取消与退出码。
fn run_with_cancel<F>(f: F) -> i32
where
    F: FnOnce(&CancelHandle) -> Result<()>,
{
    let cancel = CancelHandle::new();
    let handle = cancel.clone();
    let _ = ctrlc::set_handler(move || handle.cancel());
    match f(&cancel) {
        Ok(()) => 0,
        Err(e) => {
            if let Some(core) = e.downcast_ref::<CoreError>() {
                if matches!(core, CoreError::Cancelled) {
                    println!("已取消");
                    return 1;
                }
                eprintln!("错误: {core}");
                return exit_code(core);
            }
            eprintln!("错误: {e:#}");
            2
        }
    }
}

fn exit_code(e: &CoreError) -> i32 {
    match e {
        CoreError::Cancelled => 1,
        _ => 2,
    }
}

/// indicatif 驱动的进度回调。
struct CliProgress {
    bar: Mutex<Option<ProgressBar>>,
}

impl CliProgress {
    fn new(total: u64) -> Self {
        if total > 0 {
            let bar = ProgressBar::new(total);
            bar.set_style(
                ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% {msg}")
                    .unwrap()
                    .with_key("eta", |s: &ProgressState, w: &mut dyn fmt::Write| {
                        write!(w, "{:.1}s", s.eta().as_secs_f64()).unwrap()
                    })
                    .progress_chars("##-"),
            );
            Self {
                bar: Mutex::new(Some(bar)),
            }
        } else {
            Self {
                bar: Mutex::new(None),
            }
        }
    }
}

impl ProgressSink for CliProgress {
    fn update(&self, r: &ProgressReport) {
        let msg = format!("{} {}", phase_str(r.phase), r.current_file.display());
        if let Some(bar) = self.bar.lock().unwrap().as_ref() {
            bar.set_length(r.total_bytes.max(1));
            bar.set_position(r.done_bytes);
            bar.set_message(msg);
            if r.done_bytes >= r.total_bytes {
                bar.finish();
            }
        }
    }
}

fn phase_str(phase: Phase) -> &'static str {
    match phase {
        Phase::Listing => "列出",
        Phase::Compressing => "压缩",
        Phase::Extracting => "解压",
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}
