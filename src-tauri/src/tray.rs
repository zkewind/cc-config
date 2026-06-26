//! 托盘菜单管理模块
//!
//! 负责系统托盘图标和菜单的创建、更新和事件处理。

use tauri::menu::{CheckMenuItem, Menu, MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::{Emitter, Manager};

use crate::app_config::AppType;
use crate::error::AppError;
use crate::store::AppState;

/// 托盘菜单文本（国际化）
#[derive(Clone, Copy)]
pub struct TrayTexts {
    pub global_config: &'static str,
    pub current_project_label: &'static str,
    pub no_providers_label: &'static str,
    pub lightweight_mode: &'static str,
    pub quit: &'static str,
    pub _auto_label: &'static str,
    pub others_label: &'static str,
}

impl TrayTexts {
    pub fn from_language(language: &str) -> Self {
        match language {
            "en" => Self {
                global_config: "Global Providers",
                current_project_label: "Current Project",
                no_providers_label: "(no providers)",
                lightweight_mode: "Lightweight Mode",
                quit: "Quit",
                _auto_label: "Auto (Failover)",
                others_label: "Others",
            },
            _ => Self {
                global_config: "全局供应商",
                current_project_label: "当前项目",
                no_providers_label: "(无供应商)",
                lightweight_mode: "轻量模式",
                quit: "退出",
                _auto_label: "自动 (故障转移)",
                others_label: "其他",
            },
        }
    }
}

/// 托盘应用分区配置
pub struct TrayAppSection {
    pub app_type: AppType,
    pub prefix: &'static str,
    pub empty_id: &'static str,
    pub header_label: &'static str,
    pub log_name: &'static str,
}

pub const TRAY_ID: &str = "claude-code-multi-config";

pub const TRAY_SECTIONS: [TrayAppSection; 1] = [TrayAppSection {
    app_type: AppType::Claude,
    prefix: "claude_",
    empty_id: "claude_empty",
    header_label: "Claude",
    log_name: "Claude",
}];

/// 托盘供应商事件分类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderEventKind {
    /// 全局供应商切换，携带 provider_id
    Global(String),
    /// 当前项目供应商切换，携带 provider_id
    Project(String),
}

/// 将托盘菜单事件 id 分类为全局/项目供应商事件。
///
/// 注意：项目前缀 `<prefix>proj_` 必须先于全局前缀 `<prefix>` 判定，
/// 因为前者是后者的前缀（`claude_proj_x` 以 `claude_` 开头）。
pub fn classify_provider_event(event_id: &str) -> Option<ProviderEventKind> {
    for section in TRAY_SECTIONS.iter() {
        let project_prefix = format!("{}proj_", section.prefix);
        if let Some(pid) = event_id.strip_prefix(&project_prefix) {
            return Some(ProviderEventKind::Project(pid.to_string()));
        }
        if let Some(pid) = event_id.strip_prefix(section.prefix) {
            return Some(ProviderEventKind::Global(pid.to_string()));
        }
    }
    None
}

/// 对供应商列表排序：sort_index → created_at → name
fn sort_providers(
    providers: &indexmap::IndexMap<String, crate::provider::Provider>,
) -> Vec<(&String, &crate::provider::Provider)> {
    let mut sorted: Vec<_> = providers.iter().collect();
    sorted.sort_by(|(_, a), (_, b)| {
        match (a.sort_index, b.sort_index) {
            (Some(idx_a), Some(idx_b)) => return idx_a.cmp(&idx_b),
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        match (a.created_at, b.created_at) {
            (Some(time_a), Some(time_b)) => return time_a.cmp(&time_b),
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            _ => {}
        }

        a.name.cmp(&b.name)
    });
    sorted
}

/// 处理供应商托盘事件
pub fn handle_provider_tray_event(app: &tauri::AppHandle, event_id: &str) -> bool {
    let Some(kind) = classify_provider_event(event_id) else {
        return false;
    };
    match kind {
        ProviderEventKind::Global(provider_id) => {
            log::info!("切换到全局供应商: {provider_id}");
            let app_handle = app.clone();
            let app_type = TRAY_SECTIONS[0].app_type.clone();
            tauri::async_runtime::spawn_blocking(move || {
                if let Err(e) = handle_provider_click(&app_handle, &app_type, &provider_id) {
                    log::error!("切换全局供应商失败: {e}");
                }
            });
            true
        }
        ProviderEventKind::Project(provider_id) => {
            log::info!("切换当前项目供应商: {provider_id}");
            let app_handle = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                if let Some(app_state) = app_handle.try_state::<AppState>() {
                    if let Err(e) =
                        handle_project_provider_click(&app_handle, app_state.inner(), &provider_id)
                    {
                        log::error!("切换当前项目供应商失败: {e}");
                    }
                }
            });
            true
        }
    }
}

/// 处理供应商点击：切换供应商
fn handle_provider_click(
    app: &tauri::AppHandle,
    app_type: &AppType,
    provider_id: &str,
) -> Result<(), AppError> {
    if let Some(app_state) = app.try_state::<AppState>() {
        crate::services::ProviderService::switch(app_state.inner(), app_type.clone(), provider_id)?;

        if let Ok(new_menu) = create_tray_menu(app, app_state.inner()) {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_menu(Some(new_menu));
            }
        }

        let event_data = serde_json::json!({
            "appType": app_type.as_str(),
            "providerId": provider_id
        });
        if let Err(e) = app.emit("provider-switched", event_data) {
            log::error!("发射 provider-switched 事件失败: {e}");
        }
    }
    Ok(())
}

/// 处理当前项目供应商点击：写入项目的 `.claude/settings.json`。
///
/// 项目路径取自后端同步的前端选中作用域（`current_project_scope`），
/// 真实来源为项目实时 `.claude/settings.json` 文件。
fn handle_project_provider_click(
    app: &tauri::AppHandle,
    app_state: &AppState,
    provider_id: &str,
) -> Result<(), AppError> {
    let project_path = app_state
        .db
        .get_current_project_scope()?
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty() && *p != "user")
        .map(str::to_owned)
        .ok_or_else(|| AppError::Message("当前未选中有效项目作用域".to_string()))?;

    let provider = app_state
        .db
        .get_all_providers(AppType::Claude.as_str())?
        .get(provider_id)
        .cloned()
        .ok_or_else(|| AppError::Message(format!("供应商不存在: {provider_id}")))?;

    crate::apply_provider_to_project(&provider, &project_path)?;

    if let Ok(new_menu) = create_tray_menu(app, app_state) {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_menu(Some(new_menu));
        }
    }

    let event_data = serde_json::json!({
        "projectPath": project_path,
        "providerId": provider_id
    });
    if let Err(e) = app.emit("project-provider-switched", event_data) {
        log::error!("发射 project-provider-switched 事件失败: {e}");
    }

    Ok(())
}

/// 创建动态托盘菜单
pub fn create_tray_menu(
    app: &tauri::AppHandle,
    app_state: &AppState,
) -> Result<Menu<tauri::Wry>, AppError> {
    let app_settings = crate::settings::get_settings();
    let tray_texts = TrayTexts::from_language(app_settings.language.as_deref().unwrap_or("zh"));

    let mut menu_builder = MenuBuilder::new(app);

    for section in TRAY_SECTIONS.iter() {
        let app_type_str = section.app_type.as_str();
        let providers = app_state.db.get_all_providers(app_type_str)?;

        let current_id =
            crate::settings::get_effective_current_provider(&app_state.db, &section.app_type)?
                .unwrap_or_default();

        let global_config_id = format!("global_config_{}", app_type_str);
        let global_config_item = MenuItem::with_id(
            app,
            &global_config_id,
            tray_texts.global_config,
            false,
            None::<&str>,
        )
        .map_err(|e| AppError::Message(format!("创建全局配置标签失败: {e}")))?;
        menu_builder = menu_builder.item(&global_config_item);

        if providers.is_empty() {
            let label = format!("{} {}", section.header_label, tray_texts.no_providers_label);
            let empty_item = MenuItem::with_id(app, section.empty_id, &label, false, None::<&str>)
                .map_err(|e| {
                    AppError::Message(format!("创建{}空提示失败: {e}", section.log_name))
                })?;
            menu_builder = menu_builder.item(&empty_item);
        } else {
            let limit = crate::settings::effective_tray_provider_limit();
            let sorted = sort_providers(&providers);

            // 溢出供应商放入"其他"子菜单，置于本 section 首位
            if sorted.len() > limit {
                let others_id = format!("submenu_{}_others", app_type_str);
                let mut others = SubmenuBuilder::with_id(app, &others_id, tray_texts.others_label);
                for &(id, provider) in sorted.iter().skip(limit) {
                    let is_current = current_id == *id;
                    let item = CheckMenuItem::with_id(
                        app,
                        format!("{}{}", section.prefix, id),
                        &provider.name,
                        true,
                        is_current,
                        None::<&str>,
                    )
                    .map_err(|e| {
                        AppError::Message(format!("创建{}溢出菜单项失败: {e}", section.log_name))
                    })?;
                    others = others.item(&item);
                }
                let others_submenu = others.build().map_err(|e| {
                    AppError::Message(format!("构建{}溢出子菜单失败: {e}", section.log_name))
                })?;
                menu_builder = menu_builder.item(&others_submenu);
            }

            // 前 limit 个供应商直接塞入一级菜单
            for &(id, provider) in sorted.iter().take(limit) {
                let is_current = current_id == *id;
                let item = CheckMenuItem::with_id(
                    app,
                    format!("{}{}", section.prefix, id),
                    &provider.name,
                    true,
                    is_current,
                    None::<&str>,
                )
                .map_err(|e| {
                    AppError::Message(format!("创建{}菜单项失败: {e}", section.log_name))
                })?;
                menu_builder = menu_builder.item(&item);
            }
        }

        // ── 当前项目区块（跟随前端选中项目） ──
        let current_scope = app_state.db.get_current_project_scope()?;
        let visible_project = current_scope
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty() && *p != "user")
            .filter(|p| {
                app_state
                    .db
                    .get_managed_project_paths()
                    .unwrap_or_default()
                    .iter()
                    .any(|m| m.as_str() == *p)
            })
            .filter(|p| std::path::Path::new(*p).exists());

        if let Some(project_path) = visible_project {
            let display_name = std::path::Path::new(project_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| project_path.to_string());
            let header_label = format!("{} · {}", tray_texts.current_project_label, display_name);
            let header_item = MenuItem::with_id(
                app,
                "current_project_header",
                &header_label,
                false,
                None::<&str>,
            )
            .map_err(|e| AppError::Message(format!("创建当前项目标题失败: {e}")))?;
            menu_builder = menu_builder.separator().item(&header_item);

            let proj_current_id =
                crate::resolve_provider_from_project_internal(
                    app_state,
                    app_type_str,
                    project_path,
                )
                .unwrap_or_default();

            if providers.is_empty() {
                let label = format!(
                    "{} {}",
                    tray_texts.current_project_label, tray_texts.no_providers_label
                );
                let empty_item =
                    MenuItem::with_id(app, "current_project_empty", &label, false, None::<&str>)
                        .map_err(|e| AppError::Message(format!("创建当前项目空提示失败: {e}")))?;
                menu_builder = menu_builder.item(&empty_item);
            } else {
                let limit = crate::settings::effective_tray_provider_limit();
                let proj_sorted = sort_providers(&providers);

                if proj_sorted.len() > limit {
                    let others_id = format!("submenu_{}_proj_others", app_type_str);
                    let mut others =
                        SubmenuBuilder::with_id(app, &others_id, tray_texts.others_label);
                    for &(id, provider) in proj_sorted.iter().skip(limit) {
                        let is_current = proj_current_id == *id;
                        let item = CheckMenuItem::with_id(
                            app,
                            format!("{}proj_{}", section.prefix, id),
                            &provider.name,
                            true,
                            is_current,
                            None::<&str>,
                        )
                        .map_err(|e| {
                            AppError::Message(format!("创建当前项目溢出菜单项失败: {e}"))
                        })?;
                        others = others.item(&item);
                    }
                    let others_submenu = others
                        .build()
                        .map_err(|e| AppError::Message(format!("构建当前项目溢出子菜单失败: {e}")))?;
                    menu_builder = menu_builder.item(&others_submenu);
                }

                for &(id, provider) in proj_sorted.iter().take(limit) {
                    let is_current = proj_current_id == *id;
                    let item = CheckMenuItem::with_id(
                        app,
                        format!("{}proj_{}", section.prefix, id),
                        &provider.name,
                        true,
                        is_current,
                        None::<&str>,
                    )
                    .map_err(|e| AppError::Message(format!("创建当前项目菜单项失败: {e}")))?;
                    menu_builder = menu_builder.item(&item);
                }
            }
        }

        menu_builder = menu_builder.separator();
    }

    let lightweight_item = CheckMenuItem::with_id(
        app,
        "lightweight_mode",
        tray_texts.lightweight_mode,
        true,
        crate::lightweight::is_lightweight_mode(),
        None::<&str>,
    )
    .map_err(|e| AppError::Message(format!("创建轻量模式菜单失败: {e}")))?;

    menu_builder = menu_builder.item(&lightweight_item).separator();

    let quit_item = MenuItem::with_id(app, "quit", tray_texts.quit, true, None::<&str>)
        .map_err(|e| AppError::Message(format!("创建退出菜单失败: {e}")))?;

    menu_builder = menu_builder.item(&quit_item);

    let menu = menu_builder
        .build()
        .map_err(|e| AppError::Message(format!("构建菜单失败: {e}")))?;

    Ok(menu)
}

pub fn refresh_tray_menu(app: &tauri::AppHandle) {
    use crate::store::AppState;

    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(new_menu) = create_tray_menu(app, state.inner()) {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                if let Err(e) = tray.set_menu(Some(new_menu)) {
                    log::error!("刷新托盘菜单失败: {e}");
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub fn apply_tray_policy(app: &tauri::AppHandle, dock_visible: bool) {
    use tauri::ActivationPolicy;

    let desired_policy = if dock_visible {
        ActivationPolicy::Regular
    } else {
        ActivationPolicy::Accessory
    };

    if let Err(err) = app.set_dock_visibility(dock_visible) {
        log::warn!("设置 Dock 显示状态失败: {err}");
    }

    if let Err(err) = app.set_activation_policy(desired_policy) {
        log::warn!("设置激活策略失败: {err}");
    }
}

/// 显示并激活主窗口
///
/// 若主窗口存在则取消最小化、显示并聚焦；若处于轻量模式且主窗口不存在，则退出轻量模式重建窗口。
pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "windows")]
        {
            let _ = window.set_skip_taskbar(false);
        }
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        #[cfg(target_os = "linux")]
        {
            crate::linux_fix::nudge_main_window(window.clone());
        }
        #[cfg(target_os = "macos")]
        {
            apply_tray_policy(app, true);
        }
    } else if crate::lightweight::is_lightweight_mode() {
        if let Err(e) = crate::lightweight::exit_lightweight_mode(app) {
            log::error!("退出轻量模式重建窗口失败: {e}");
        }
    }
}

/// 将主窗口隐藏到系统托盘
pub fn hide_main_window_to_tray(window: &tauri::WebviewWindow) {
    let _ = window.hide();
    #[cfg(target_os = "windows")]
    {
        let _ = window.set_skip_taskbar(true);
    }
    #[cfg(target_os = "macos")]
    {
        apply_tray_policy(window.app_handle(), false);
    }
}

/// 处理托盘图标点击事件
///
/// - 左键单击 / 双击：打开主窗口
/// - 右键单击：弹出托盘菜单（由系统默认行为触发，无需在此处理）
pub fn handle_tray_icon_event(app: &tauri::AppHandle, event: &tauri::tray::TrayIconEvent) {
    use tauri::tray::{MouseButton, TrayIconEvent};

    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            ..
        } => {
            log::info!("托盘左键单击：打开主窗口");
            show_main_window(app);
        }
        TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            log::info!("托盘左键双击：打开主窗口");
            show_main_window(app);
        }
        _ => {
            log::debug!("未处理的托盘图标事件: {event:?}");
        }
    }
}

/// 处理托盘菜单事件
pub fn handle_tray_menu_event(app: &tauri::AppHandle, event_id: &str) {
    log::info!("处理托盘菜单事件: {event_id}");

    match event_id {
        "lightweight_mode" => {
            if crate::lightweight::is_lightweight_mode() {
                if let Err(e) = crate::lightweight::exit_lightweight_mode(app) {
                    log::error!("退出轻量模式失败: {e}");
                }
            } else if let Err(e) = crate::lightweight::enter_lightweight_mode(app) {
                log::error!("进入轻量模式失败: {e}");
            }
        }
        "quit" => {
            log::info!("退出应用");
            app.exit(0);
        }
        _ => {
            if handle_provider_tray_event(app, event_id) {
                return;
            }
            log::warn!("未处理的菜单事件: {event_id}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TRAY_ID;

    #[test]
    fn tray_id_is_unique_to_app() {
        assert_eq!(TRAY_ID, "claude-code-multi-config");
        assert_ne!(TRAY_ID, "main");
    }

    #[test]
    fn classify_distinguishes_global_and_project_events() {
        use super::{classify_provider_event, ProviderEventKind};
        assert_eq!(
            classify_provider_event("claude_p1"),
            Some(ProviderEventKind::Global("p1".to_string()))
        );
        assert_eq!(
            classify_provider_event("claude_proj_p2"),
            Some(ProviderEventKind::Project("p2".to_string()))
        );
        assert_eq!(classify_provider_event("lightweight_mode"), None);
        assert_eq!(classify_provider_event("quit"), None);
        assert_eq!(classify_provider_event("current_project_header"), None);
    }
}
