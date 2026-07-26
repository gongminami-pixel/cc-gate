//! macOS menubar (system tray) integration.

use std::sync::Arc;
use std::time::Duration;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
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
        .menu(&menu)
        .icon_as_template(false)
        .show_menu_on_left_click(true)
        .tooltip("CC-Gate")
        .on_menu_event(move |app, event| handle_menu_event(app, event.id.as_ref()))
        .build(app)?;

    // Background refresh: icon+tooltip every 5s, menu only on change
    let app_for_refresh = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut last_summary: String = String::new();
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            refresh(&app_for_refresh, &mut last_summary).await;
        }
    });

    Ok(())
}

pub async fn refresh(app: &AppHandle, last_summary: &mut String) {
    let manager = match app.try_state::<Arc<ProxyManager>>() {
        Some(m) => m.inner().clone(),
        None => return,
    };
    let statuses = manager.status_all().await;
    // Update icon + tooltip only; skip menu rebuild unless summary changed
    update_icon_and_tooltip(app, &statuses).await;
    rebuild_menu_if_changed(app, &statuses, last_summary).await;
}

/// Force-full refresh (used after toggle/restart from tray menu or initial setup)
pub async fn force_refresh(app: &AppHandle) {
    let manager = match app.try_state::<Arc<ProxyManager>>() {
        Some(m) => m.inner().clone(),
        None => return,
    };
    let statuses = manager.status_all().await;
    update_icon_and_tooltip(app, &statuses).await;
    let summary_text = build_summary(&statuses);
    rebuild_menu_always(app, &statuses, &summary_text).await;
}

async fn update_icon_and_tooltip(app: &AppHandle, statuses: &[crate::types::ProxyStatus]) {
    let active = statuses.iter().filter(|s| s.running).count();
    let total = statuses.len();
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(format!("CC-Gate · 活跃 {active} / 共 {total}")));
    }
}

async fn rebuild_menu_if_changed(app: &AppHandle, statuses: &[crate::types::ProxyStatus], last_summary: &mut String) {
    let summary_text = build_summary(statuses);
    if summary_text == *last_summary { return; }
    *last_summary = summary_text.clone();
    rebuild_menu_always(app, statuses, &summary_text).await;
}

fn build_summary(statuses: &[crate::types::ProxyStatus]) -> String {
    let active = statuses.iter().filter(|s| s.running).count();
    let total = statuses.len();
    format!("CC-Gate · 代理活跃 {active} / 共 {total}")
}

async fn rebuild_menu_always(app: &AppHandle, statuses: &[crate::types::ProxyStatus], summary_text: &str) {
    let active = statuses.iter().filter(|s| s.running).count();
    let total = statuses.len();
    let app_clone = app.clone();
    let statuses_clone = statuses.to_vec();
    let summary = summary_text.to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();
    if app.run_on_main_thread(move || {
        let _ = tx.send(build_menu(&app_clone, &statuses_clone, active, total, &summary).ok());
    }).is_err() {
        return;
    }
    let menu = match rx.await.ok().flatten() {
        Some(m) => m,
        None => { tracing::warn!("tray refresh: menu rebuild failed"); return; }
    };
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_menu(Some(menu));
    }
}

fn build_menu(
    app: &AppHandle,
    statuses: &[crate::types::ProxyStatus],
    _active: usize,
    _total: usize,
    summary_text: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    let summary = summary_text.to_string();

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
                    force_refresh(&app).await;
                }
            });
        }
        "restart" => {
            tauri::async_runtime::spawn(async move {
                if let Some(mgr) = app.try_state::<Arc<ProxyManager>>() {
                    let m = mgr.inner().clone();
                    let (port, script) = proxy_defaults(&proxy_name);
                    let _ = m.restart(&proxy_name, port, &script).await;
                    force_refresh(&app).await;
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
