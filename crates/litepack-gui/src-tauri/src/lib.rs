pub mod commands;
pub mod context_menu;
pub mod state;

use std::sync::Arc;

use state::{PendingAction, PendingActionState, PendingActionType};

fn pending_action_from_args() -> Option<PendingAction> {
    let args: Vec<String> = std::env::args().collect();
    log::info!("启动参数: {:?}", args);
    let mut iter = args.into_iter().skip(1);
    while let Some(arg) = iter.next() {
        let action_type = match arg.as_str() {
            "--open" => Some(PendingActionType::Open),
            "--extract-here" => Some(PendingActionType::ExtractHere),
            "--extract-to" => Some(PendingActionType::ExtractTo),
            "--extract-named" => Some(PendingActionType::ExtractNamed),
            _ => continue,
        };
        if let Some(path) = iter.next() {
            log::info!("解析到操作: {:?}, 路径: {}", action_type, path);
            return Some(PendingAction {
                action: action_type.unwrap(),
                path,
            });
        }
    }
    log::info!("无待处理操作");
    None
}

/// 根据操作类型和路径计算前端路由。
fn action_to_route(action: &PendingAction) -> String {
    let encoded = urlencoding::encode(&action.path);
    let route = match action.action {
        PendingActionType::Open => "/open",
        PendingActionType::ExtractHere => "/extract-here",
        PendingActionType::ExtractTo => "/extract-to",
        PendingActionType::ExtractNamed => "/extract-named",
    };
    format!("{route}?path={encoded}")
}

/// 让窗口跳转到指定路由。
fn navigate_to(window: &tauri::WebviewWindow, route: &str) {
    let resolved = window
        .url()
        .ok()
        .and_then(|base| base.join(route).ok())
        .or_else(|| tauri::Url::parse(&format!("tauri://localhost{route}")).ok());
    match resolved {
        Some(url) => {
            log::info!("跳转到: {}", url);
            if let Err(e) = window.navigate(url) {
                log::error!("窗口跳转失败: {:?}", e);
            }
        }
        None => {
            log::error!("解析路由失败: {}", route);
        }
    }
}

fn use_extract_window(action: &PendingAction) -> bool {
    matches!(
        action.action,
        PendingActionType::ExtractHere | PendingActionType::ExtractNamed
    )
}

/// 创建窗口并等页面加载完成后导航。
fn create_window_with_route(
    app: &tauri::App,
    label: &str,
    title: &str,
    route: &str,
    width: f64,
    height: f64,
    resizable: bool,
) {
    let route_clone = route.to_string();
    let label_clone = label.to_string();
    let app_handle = app.handle().clone();
    let navigated = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let navigated_clone = navigated.clone();
    let _window = tauri::WebviewWindowBuilder::new(
        &app_handle,
        label,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(title)
    .inner_size(width, height)
    .resizable(resizable)
    .decorations(false)
    .center()
    .on_page_load(move |window, payload| {
        use tauri::webview::PageLoadEvent;
        if payload.event() == PageLoadEvent::Finished
            && !navigated_clone.swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            log::info!("[{}] 页面加载完成，开始导航", label_clone);
            navigate_to(&window, &route_clone);
            window.show().ok();
            window.set_focus().ok();
        }
    })
    .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_plugin = {
        let log_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("logs")))
            .unwrap_or_default();
        tauri_plugin_log::Builder::new()
            .targets([
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                    path: log_dir,
                    file_name: None,
                }),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
            ])
            .max_file_size(1_000_000)
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(6))
            .level(log::LevelFilter::Info)
            .build()
    };

    let pending = pending_action_from_args();
    let is_extract = pending.as_ref().map(use_extract_window).unwrap_or(false);

    let mut builder = tauri::Builder::default()
        .plugin(log_plugin)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::new(state::OperationRegistry::new()))
        .manage(PendingActionState(pending.clone().into()));

    builder = builder.setup(move |app| {
        log::info!("setup 回调触发");

        if let Some(action) = pending {
            let route = action_to_route(&action);
            log::info!("action={:?}, route={}", action.action, route);

            if is_extract {
                create_window_with_route(
                    app,
                    "extract",
                    "LitePack - 解压中",
                    &route,
                    400.0,
                    150.0,
                    false,
                );
            } else if action.action == PendingActionType::Open {
                create_window_with_route(app, "main", "LitePack", &route, 800.0, 580.0, true);
            } else {
                create_window_with_route(app, "main", "LitePack", &route, 480.0, 240.0, true);
            }
        } else {
            create_window_with_route(app, "main", "LitePack", "/", 960.0, 700.0, true);
        }
        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![
            commands::compress_files,
            commands::extract_archive,
            commands::list_archive,
            commands::cancel_operation,
            commands::register_context_menu,
            commands::unregister_context_menu,
            commands::context_menu_status,
            commands::take_pending_action,
            commands::take_pending_extract,
            commands::exit_app,
            commands::open_extract_password_window,
            commands::normalize_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
