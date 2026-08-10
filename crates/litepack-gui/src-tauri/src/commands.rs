use std::path::{Path, PathBuf};
use std::sync::Arc;

use litepack_core::{
    collect_entries, CancelHandle, CompressOptions, EntryInfo, ExtractOptions, OverwriteMode,
    Phase, ProgressReport, ProgressSink,
};
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::State;

use crate::{context_menu, navigate_to};
use crate::state::{OperationRegistry, PendingAction, PendingActionState};

/// 通过 Channel 推送的事件：进度 / 完成 / 出错。
#[derive(Clone, Debug, Serialize)]
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
    eprintln!("[extract_archive] archive={archive} out_dir={out_dir} overwrite={overwrite}");
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
        eprintln!("[extract_archive] spawn_blocking started, id={id}");
        let result = litepack_core::decompress(
            Path::new(&archive),
            Path::new(&out_dir),
            &opts,
            &sink,
            &cancel,
        );
        eprintln!("[extract_archive] decompress result: {result:?}");
        registry.unregister(id);
        let event = match result {
            Ok(()) => ChannelEvent::Done { id },
            Err(e) => ChannelEvent::Error {
                id,
                message: e.to_string(),
            },
        };
        eprintln!("[extract_archive] sending event: {event:?}");
        let _ = sink.channel.send(event);
        eprintln!("[extract_archive] event sent");
    });
    Ok(id)
}

#[tauri::command]
pub(crate) async fn list_archive(
    archive: String,
    password: Option<String>,
) -> Result<Vec<EntryInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        litepack_core::list_with_password(Path::new(&archive), password.as_deref())
    })
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

/// 读取并消耗待处理的右键菜单操作（返回操作类型和路径）。
#[tauri::command]
pub(crate) fn take_pending_action(state: State<'_, PendingActionState>) -> Option<PendingAction> {
    state.0.lock().take()
}

/// 向后兼容：仅返回待处理的归档路径（旧前端使用）。
#[tauri::command]
pub(crate) fn take_pending_extract(state: State<'_, PendingActionState>) -> Option<String> {
    state.0.lock().take().map(|a| a.path)
}

#[tauri::command]
pub(crate) fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub(crate) async fn open_extract_password_window(
    app: tauri::AppHandle,
    error_message: String,
) -> Result<(), String> {
    let route = format!(
        "/extract-password?error={}",
        urlencoding::encode(&error_message)
    );
    let route_clone = route.clone();
    let navigated = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let navigated_clone = navigated.clone();

    let _window = tauri::WebviewWindowBuilder::new(
        &app,
        "extract-password",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("LitePack - 输入密码")
    .inner_size(400.0, 220.0)
    .resizable(false)
    .decorations(true)
    .center()
    .on_page_load(move |window, payload| {
        use tauri::webview::PageLoadEvent;
        if payload.event() == PageLoadEvent::Finished
            && !navigated_clone.swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            log::info!("[extract-password] 页面加载完成，开始导航");
            navigate_to(&window, &route_clone);
            let _ = window.show();
            let _ = window.set_focus();
        }
    })
    .build()
    .map_err(|e| format!("创建密码窗口失败: {e}"))?;

    Ok(())
}

fn validate_target(target: &str, format: &str) -> Result<(), String> {
    let ext = Path::new(target)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    let expected: String = if format.is_empty() {
        ext.clone()
            .ok_or_else(|| "输出路径缺少扩展名，无法推断格式".to_string())?
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

/// 将 Windows 反斜杠路径转换为正斜杠路径。
#[tauri::command]
pub(crate) fn normalize_path(path: String) -> String {
    use path_slash::PathExt;
    std::path::Path::new(&path).to_slash_lossy().into_owned()
}
