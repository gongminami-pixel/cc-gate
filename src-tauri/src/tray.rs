//! macOS menubar (system tray) integration.

use std::sync::Arc;
use std::time::Duration;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::proxy_manager::ProxyManager;

const TRAY_ID: &str = "main";

// 16x16 tray icon — embedded at compile time
fn load_tray_icon() -> Option<Image<'static>> {
    let data = include_bytes!("../icons/tray-icon.png");
    let img = image::load_from_memory(data).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Some(Image::new_owned(rgba.into_raw(), w, h))
}

pub fn setup(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_tray_icon()
        .unwrap_or_else(|| Image::new_owned(vec![0; 16*16*4], 16, 16));

    let summary = MenuItem::with_id(app, "summary", "CC-Gate · 加载中…", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 CC-Gate", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &summary,
            &PredefinedMenuItem::separator(app)?,
            &show,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("CC-Gate")
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .on_menu_event(move |app, event| handle_menu_event(app, event.id.as_ref()))
        .build(app)?;

    // Background refresh every 5s
    let app_for_refresh = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            refresh(&app_for_refresh).await;
        }
    });

    Ok(())
}

pub async fn refresh(app: &AppHandle) {
    let manager = match app.try_state::<Arc<ProxyManager>>() {
        Some(m) => m.inner().clone(),
        None => return,
    };
    let statuses = manager.status_all().await;
    let active = statuses.iter().filter(|s| s.running).count();
    let total = statuses.len();

    // Build menu
    let app_clone = app.clone();
    let statuses_clone = statuses.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    if app.run_on_main_thread(move || {
        let _ = tx.send(build_menu(&app_clone, &statuses_clone, active, total).ok());
    }).is_err() {
        return;
    }
    let menu = match rx.await.ok().flatten() {
        Some(m) => m,
        None => {
            tracing::warn!("tray refresh: menu rebuild failed");
            return;
        }
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_menu(Some(menu));
        if let Some(icon) = load_tray_icon() {
            let _ = tray.set_icon(Some(icon));
            let _ = tray.set_icon_as_template(true);
        }
        let tip = format!("CC-Gate · 活跃 {active} / 共 {total}");
        let _ = tray.set_tooltip(Some(tip));
    }
}

fn build_menu(
    app: &AppHandle,
    statuses: &[crate::types::ProxyStatus],
    active: usize,
    total: usize,
) -> tauri::Result<Menu<tauri::Wry>> {
    let summary = format!("CC-Gate · 代理活跃 {active} / 共 {total}");

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = vec![
        Box::new(MenuItem::with_id(app, "summary", &summary, false, None::<&str>)?),
        Box::new(PredefinedMenuItem::separator(app)?),
    ];

    for s in statuses {
        let dot = if s.running { "●" } else { "○" };
        let label = match (s.running, s.pid) {
            (true, Some(pid)) => format!("{dot} {} :{} · pid {pid}", s.name, s.port),
            (false, _) => format!("{dot} {} :{} · 已停止", s.name, s.port),
            _ => format!("{dot} {} :{} · 启动中…", s.name, s.port),
        };

        let toggle_label = if s.running { "��止" } else { "启动" };
        let toggle = MenuItem::with_id(
            app,
            format!("toggle:{}", s.name),
            toggle_label,
            true,
            None::<&str>,
        )?;
        let restart = MenuItem::with_id(
            app,
            format!("restart:{}", s.name),
            "重启",
            s.running,
            None::<&str>,
        )?;
        let sub = tauri::menu::Submenu::with_items(
            app, &label, true, &[&toggle, &restart],
        )?;
        items.push(Box::new(sub));
    }

    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, "quit", "退出 CC-Gate", true, None::<&str>)?));

    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &item_refs)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    if id == "summary" { return; }
    if id == "show" {
        show_main_window(app);
        return;
    }
    if id == "quit" {
        app.exit(0);
        return;
    }

    let parts: Vec<&str> = id.splitn(2, ':').collect();
    if parts.len() != 2 { return; }
    let action = parts[0];
    let proxy_name = parts[1].to_string();
    let app = app.clone();

    match action {
        "toggle" => {
            tauri::async_runtime::spawn(async move {
                if let Some(mgr) = app.try_state::<Arc<ProxyManager>>() {
                    let m = mgr.inner().clone();
                    let statuses = m.status_all().await;
                    let target = statuses.iter().find(|s| s.name == proxy_name);
                    if let Some(s) = target {
                        if s.running {
                            let _ = m.stop(&proxy_name).await;
                        } else {
                            let (port, script) = proxy_defaults(&proxy_name);
                            let _ = m.start(&proxy_name, port, &script).await;
                        }
                    }
                    refresh(&app).await;
                }
            });
        }
        "restart" => {
            tauri::async_runtime::spawn(async move {
                if let Some(mgr) = app.try_state::<Arc<ProxyManager>>() {
                    let m = mgr.inner().clone();
                    let (port, script) = proxy_defaults(&proxy_name);
                    let _ = m.restart(&proxy_name, port, &script).await;
                    refresh(&app).await;
                }
            });
        }
        _ => {}
    }
}

fn proxy_defaults(name: &str) -> (u16, String) {
    match name {
        "mimo2codex" => (8688, "mimo2codex".into()),
        "claude-proxy" => {
            let p = crate::paths::mimo2codex_dir().join("claude-proxy.js");
            (8689, p.to_string_lossy().to_string())
        }
        "chat-proxy" => {
            let p = crate::paths::mimo2codex_dir().join("chat-proxy.js");
            (8690, p.to_string_lossy().to_string())
        }
        _ => (0, String::new()),
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if !window.is_visible().unwrap_or(false) {
            let _ = window.show();
        }
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
