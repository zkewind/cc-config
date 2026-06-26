/// 命令注册宏
///
/// 将 tauri::generate_handler![] 中所有命令按业务模块分组，
/// 通过 all_handlers!() 宏统一供 lib.rs 调用，减少 lib.rs 体积。
///
/// 分组说明：
///   provider      - 供应商管理（增删改查、切换、导入、配置）
///   mcp           - MCP 服务器管理（旧/新两套 API）
///   settings      - 应用设置与日志配置
///   plugin        - Claude 插件与 Onboarding
///   usage         - 使用量统计、Stream 健康检查、会话同步
///   subscription  - 订阅配额、余额查询
///   prompt        - 提示词管理
///   skill         - 技能管理（v3.10.0+ 统一 + 旧版兼容）
///   session       - 会话管理
///   sync          - 配置导入导出、WebDAV 同步、数据库备份
///   deeplink      - 深链接解析与导入
///   env           - 环境变量管理
///   misc          - 杂项工具（剪贴板、窗口主题、终端、版本检测等）
///   tray          - 系统托盘菜单更新
///   lightweight   - 轻量模式
///
/// 使用方式（在 lib.rs 中）：
/// ```rust
/// .invoke_handler(all_handlers!())
/// ```

/// 汇总所有模块的 Tauri 命令处理器。
///
/// 由于 `tauri::generate_handler!` 返回带有运行时泛型参数的闭包，
/// 不能直接用函数封装，改用宏展开方式让调用方传入正确的运行时类型。
#[macro_export]
macro_rules! all_handlers {
    () => {
        tauri::generate_handler![
            // ── Provider 供应商管理 ─────────────────────────────────────
            crate::commands::get_providers,
            crate::commands::get_current_provider,
            crate::commands::add_provider,
            crate::commands::update_provider,
            crate::commands::delete_provider,
            crate::commands::remove_provider_from_live_config,
            crate::commands::switch_provider,
            crate::commands::import_default_config,
            crate::commands::read_live_provider_settings,
            crate::commands::get_claude_common_config_snippet,
            crate::commands::set_claude_common_config_snippet,
            crate::commands::get_common_config_snippet,
            crate::commands::set_common_config_snippet,
            crate::commands::extract_common_config_snippet,
            crate::commands::update_providers_sort_order,
            crate::commands::sync_current_providers_live,
            crate::commands::switch_provider_for_project,
            crate::commands::get_current_provider_for_project,
            crate::commands::import_project_provider,
            crate::commands::resolve_current_provider_for_project,
            crate::commands::regenerate_provider_name,

            // ── Config / File 配置文件与路径 ───────────────────────────
            crate::commands::read_project_settings,
            crate::commands::save_project_settings,
            crate::commands::get_claude_config_status,
            crate::commands::get_config_status,
            crate::commands::get_claude_code_config_path,
            crate::commands::get_config_dir,
            crate::commands::open_config_folder,
            crate::commands::get_app_config_path,
            crate::commands::open_app_config_folder,
            crate::commands::get_app_config_dir_override,
            crate::commands::set_app_config_dir_override,

            // ── MCP 服务器管理 ─────────────────────────────────────────
            // 旧版 Claude MCP（claude_mcp.json 直接操作）
            crate::commands::get_claude_mcp_status,
            crate::commands::read_claude_mcp_config,
            crate::commands::upsert_claude_mcp_server,
            crate::commands::delete_claude_mcp_server,
            crate::commands::validate_mcp_command,
            // 新版 MCP（config.json SSOT）
            crate::commands::get_mcp_config,
            crate::commands::upsert_mcp_server_in_config,
            crate::commands::delete_mcp_server_in_config,
            crate::commands::set_mcp_enabled,
            // 统一 MCP 管理
            crate::commands::get_mcp_servers,
            crate::commands::upsert_mcp_server,
            crate::commands::delete_mcp_server,
            crate::commands::toggle_mcp_app,
            crate::commands::import_mcp_from_apps,

            // ── Settings 应用设置 ──────────────────────────────────────
            crate::commands::get_settings,
            crate::commands::save_settings,
            crate::commands::get_rectifier_config,
            crate::commands::set_rectifier_config,
            crate::commands::get_optimizer_config,
            crate::commands::set_optimizer_config,
            crate::commands::get_copilot_optimizer_config,
            crate::commands::set_copilot_optimizer_config,
            crate::commands::get_log_config,
            crate::commands::set_log_config,

            // ── Plugin Claude 插件 & Onboarding ───────────────────────
            crate::commands::get_claude_plugin_status,
            crate::commands::read_claude_plugin_config,
            crate::commands::apply_claude_plugin_config,
            crate::commands::is_claude_plugin_applied,
            crate::commands::apply_claude_onboarding_skip,
            crate::commands::clear_claude_onboarding_skip,

            // ── Usage Script 余额查询脚本 ──────────────────────────────
            crate::commands::queryProviderUsage,
            crate::commands::testUsageScript,
            // Stream 健康检查
            crate::commands::stream_check_provider,
            crate::commands::stream_check_all_providers,
            crate::commands::get_stream_check_config,
            crate::commands::save_stream_check_config,

            // ── Subscription 订阅 & 余额 ───────────────────────────────
            crate::commands::get_subscription_quota,
            crate::commands::get_coding_plan_quota,
            crate::commands::get_balance,

            // ── Prompt 提示词管理 ──────────────────────────────────────
            crate::commands::get_prompts,
            crate::commands::upsert_prompt,
            crate::commands::delete_prompt,
            crate::commands::enable_prompt,
            crate::commands::import_prompt_from_file,
            crate::commands::get_current_prompt_file_content,

            // ── Model 模型获取 ─────────────────────────────────────────
            crate::commands::fetch_models_for_config,

            // ── Endpoint 自定义端点 & 速度测试 ────────────────────────
            crate::commands::test_api_endpoints,
            crate::commands::get_custom_endpoints,
            crate::commands::add_custom_endpoint,
            crate::commands::remove_custom_endpoint,
            crate::commands::update_endpoint_last_used,

            // ── Skill 技能管理（v3.10.0+ 统一 API）────────────────────
            crate::commands::get_installed_skills,
            crate::commands::get_skill_backups,
            crate::commands::delete_skill_backup,
            crate::commands::install_skill_unified,
            crate::commands::uninstall_skill_unified,
            crate::commands::restore_skill_backup,
            crate::commands::toggle_skill_app,
            crate::commands::scan_unmanaged_skills,
            crate::commands::import_skills_from_apps,
            crate::commands::discover_available_skills,
            crate::commands::check_skill_updates,
            crate::commands::update_skill,
            crate::commands::migrate_skill_storage,
            crate::commands::search_skills_sh,
            // 旧版兼容 API
            crate::commands::get_skills,
            crate::commands::get_skills_for_app,
            crate::commands::install_skill,
            crate::commands::install_skill_for_app,
            crate::commands::uninstall_skill,
            crate::commands::uninstall_skill_for_app,
            crate::commands::get_skill_repos,
            crate::commands::add_skill_repo,
            crate::commands::remove_skill_repo,
            crate::commands::install_skills_from_zip,

            // ── Auto Launch 开机自启 ───────────────────────────────────
            crate::commands::set_auto_launch,
            crate::commands::get_auto_launch_status,

            // ── Project Paths 项目路径管理 ────────────────────────────
            crate::commands::get_managed_project_paths,
            crate::commands::add_managed_project_path,
            crate::commands::remove_managed_project_path,
            crate::commands::set_current_project_scope,

            // ── Session Manager 会话管理 ───────────────────────────────
            crate::commands::list_sessions,
            crate::commands::get_session_messages,
            crate::commands::delete_session,
            crate::commands::delete_sessions,
            crate::commands::launch_session_terminal,
            crate::commands::get_tool_versions,

            // ── Import / Export & WebDAV 导入导出 & 云同步 ────────────
            crate::commands::export_config_to_file,
            crate::commands::import_config_from_file,
            crate::commands::webdav_test_connection,
            crate::commands::webdav_sync_upload,
            crate::commands::webdav_sync_download,
            crate::commands::webdav_sync_save_settings,
            crate::commands::webdav_sync_fetch_remote_info,
            crate::commands::save_file_dialog,
            crate::commands::save_json_file_dialog,
            crate::commands::open_file_dialog,
            crate::commands::open_zip_file_dialog,
            crate::commands::export_providers_json,
            // 数据库备份
            crate::commands::create_db_backup,
            crate::commands::list_db_backups,
            crate::commands::restore_db_backup,
            crate::commands::rename_db_backup,
            crate::commands::delete_db_backup,

            // ── Deep Link 深链接 ───────────────────────────────────────
            crate::commands::parse_deeplink,
            crate::commands::merge_deeplink_config,
            crate::commands::import_from_deeplink,
            crate::commands::import_from_deeplink_unified,

            // ── Env 环境变量管理 ───────────────────────────────────────
            crate::commands::check_env_conflicts,
            crate::commands::delete_env_vars,
            crate::commands::restore_env_backup,

            // ── Lightweight Mode 轻量模式 ─────────────────────────────
            crate::commands::enter_lightweight_mode,
            crate::commands::exit_lightweight_mode,
            crate::commands::is_lightweight_mode,

            // ── Misc 杂项工具 ──────────────────────────────────────────
            crate::commands::get_init_error,
            crate::commands::get_migration_result,
            crate::commands::get_skills_migration_result,
            crate::commands::pick_directory,
            crate::commands::open_external,
            crate::commands::copy_text_to_clipboard,
            crate::commands::check_for_updates,
            crate::commands::is_portable_mode,
            crate::commands::restart_app,
            crate::commands::open_provider_terminal,
            crate::commands::minimize_to_tray,
            crate::commands::set_window_theme,

            // ── Tray 系统托盘 ─────────────────────────────────────────
            crate::update_tray_menu,
        ]
    };
}
