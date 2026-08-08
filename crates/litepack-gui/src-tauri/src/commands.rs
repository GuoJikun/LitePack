use std::path::{Path, PathBuf};
use std::sync::Arc;

use litepack_core::{
    collect_entries, CancelHandle, CompressOptions, EntryInfo, ExtractOptions, OverwriteMode, Phase,
    ProgressReport, ProgressSink,
};
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::State;

use crate::context_menu;
use crate::state::{OperationRegistry, PendingExtract};

/// 通过 Channel 推送的事件：进度 / 完成 / 出错。
#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ChannelEvent {
    Progress(ProgressReport),
    Done { id: u64 },
    Error { id: u64, message: String },
}

/// 将核心库进度回调桥接到 Tauri Channel。
struct ChannelSink {
    channel: Channel<ChannelEvent>,
}

impl ProgressSink for ChannelSink {
    fn update(&self, report: &ProgressReport) {
        let _ = self.channel.send(ChannelEvent::Progress(report.clone()));
    }
}

fn err_msg(e: impl ToString) -> String {
    e.to_string()
}

fn to_path(paths: Vec<String>) -> Vec<PathBuf> {
    paths.into_iter().map(PathBuf::from).collect()
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn compress_files(
    state: State<'_, Arc<OperationRegistry>>,
    paths: Vec<String>,
    target: String,
    format: String,
    level: u32,
    password: Option<String>,
    skip_hidden: bool,
    overwrite: bool,
    progress: Channel<ChannelEvent>,
) -> Result<u64, String> {
    validate_target(&target, &format)?;

    let registry = state.inner().clone();
    let opts = CompressOptions {
        level: level.min(9),
        password,
        overwrite,
        skip_hidden,
    };
    let cancel = CancelHandle::new();
    let id = registry.register(cancel.clone());
    let sink = ChannelSink { channel: progress };

    tauri::async_runtime::spawn_blocking(move || {
        let result = (|| -> litepack_core::Result<()> {
            let entries = collect_entries(&to_path(paths), &opts)?;
            let total: u64 = entries
                .iter()
                .filter(|e| !e.name.ends_with('/'))
                .filter_map(|e| e.src.metadata().ok())
                .map(|m| m.len())
                .sum();
            let _ = sink.channel.send(ChannelEvent::Progress(ProgressReport {
                phase: Phase::Compressing,
                current_file: PathBuf::from(&target),
                done_bytes: 0,
                total_bytes: total,
            }));
            litepack_core::compress(&entries, Path::new(&target), &opts, &sink, &cancel)
        })();
        registry.unregister(id);
        let event = match result {
            Ok(()) => ChannelEvent::Done { id },
            Err(e) => ChannelEvent::Error {
                id,
                message: e.to_string(),
            },
        };
        let _ = sink.channel.send(event);
    });
    Ok(id)
}

#[tauri::command]
pub(crate) async fn extract_archive(
    state: State<'_, Arc<OperationRegistry>>,
    archive: String,
    out_dir: String,
    password: Option<String>,
    overwrite: bool,
    progress: Channel<ChannelEvent>,
) -> Result<u64, String> {
    let opts = ExtractOptions {
        password,
        overwrite: if overwrite {
            OverwriteMode::Overwrite
        } else {
            OverwriteMode::Skip
        },
    };
    let registry = state.inner().clone();
    let cancel = CancelHandle::new();
    let id = registry.register(cancel.clone());
    let sink = ChannelSink { channel: progress };

    tauri::async_runtime::spawn_blocking(move || {
        let result = litepack_core::decompress(
            Path::new(&archive),
            Path::new(&out_dir),
            &opts,
            &sink,
            &cancel,
        );
        registry.unregister(id);
        let event = match result {
            Ok(()) => ChannelEvent::Done { id },
            Err(e) => ChannelEvent::Error {
                id,
                message: e.to_string(),
            },
        };
        let _ = sink.channel.send(event);
    });
    Ok(id)
}

#[tauri::command]
pub(crate) async fn list_archive(archive: String) -> Result<Vec<EntryInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || litepack_core::list(Path::new(&archive)))
        .await
        .map_err(err_msg)?
        .map_err(err_msg)
}

#[tauri::command]
pub(crate) fn cancel_operation(
    state: State<'_, Arc<OperationRegistry>>,
    id: u64,
) -> Result<(), String> {
    if state.cancel(id) {
        Ok(())
    } else {
        Err("操作不存在或已结束".to_string())
    }
}

#[tauri::command]
pub(crate) fn register_context_menu() -> Result<(), String> {
    context_menu::register()
}

#[tauri::command]
pub(crate) fn unregister_context_menu() -> Result<(), String> {
    context_menu::unregister()
}

#[tauri::command]
pub(crate) fn context_menu_status() -> bool {
    context_menu::is_registered()
}

#[tauri::command]
pub(crate) fn take_pending_extract(state: State<'_, PendingExtract>) -> Option<String> {
    state.0.lock().take()
}

#[tauri::command]
pub(crate) fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn validate_target(target: &str, format: &str) -> Result<(), String> {
    let ext = Path::new(target)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    let expected: String = if format.is_empty() {
        ext.clone().ok_or_else(|| "输出路径缺少扩展名，无法推断格式".to_string())?
    } else {
        format.to_string()
    };
    if ext.as_deref() != Some(expected.as_str()) {
        return Err(format!(
            "输出扩展名 {} 与格式 {expected} 不一致",
            ext.as_deref().unwrap_or("(无)")
        ));
    }
    if expected != "zip" && expected != "7z" {
        return Err(format!("不支持的格式: {expected}"));
    }
    Ok(())
}
