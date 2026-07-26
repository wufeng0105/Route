use route_core::{config_paths, parsers, routes, writer};
use serde::Serialize;
use std::process::Command;

/// 工具状态（返回给前端）
#[derive(Debug, Clone, Serialize)]
struct ToolStatus {
    id: String,
    name: String,
    config_exists: bool,
    current_url: Option<String>,
    format: String,
    error: Option<String>,
    config_dir: String,
    config_file: String,
    auth_file: Option<String>,
}

/// 切换结果（返回给前端）
#[derive(Debug, Clone, Serialize)]
struct SwitchResultDto {
    success: bool,
    tool_name: String,
    target_url: String,
    backup_path: Option<String>,
    replaced_count: usize,
    base_url_found: bool,
    error: Option<String>,
}

/// 获取所有工具状态
#[tauri::command]
fn get_tool_statuses() -> Vec<ToolStatus> {
    let tools_config = routes::load_tools();
    tools_config
        .tools
        .iter()
        .map(|tool| {
            let config_path = config_paths::get_tool_config_path(
                &tool.config_dir,
                &tool.config_file,
            );

            let (config_exists, current_url, error) = match &config_path {
                None => (false, None, Some("无法确定 Home 目录".to_string())),
                Some(path) => {
                    if !path.exists() {
                        (false, None, None)
                    } else {
                        match parsers::ConfigFormat::from_str(&tool.format) {
                            Some(fmt) => match writer::read_base_url(path, fmt) {
                                Ok(url) => (true, url, None),
                                Err(e) => (true, None, Some(e)),
                            },
                            None => (
                                true,
                                None,
                                Some(format!("未知配置格式: {}", tool.format)),
                            ),
                        }
                    }
                }
            };

            ToolStatus {
                id: tool.id.clone(),
                name: tool.name.clone(),
                config_exists,
                current_url,
                format: tool.format.clone(),
                error,
                config_dir: tool.config_dir.clone(),
                config_file: tool.config_file.clone(),
                auth_file: tool.auth_file.clone(),
            }
        })
        .collect()
}

/// 获取用户配置（预设 + 自定义线路）
#[tauri::command]
fn get_user_config() -> routes::UserConfig {
    routes::load_user_config()
}

/// 执行线路切换
#[tauri::command]
fn switch_route(tool_id: String, target_url: String) -> SwitchResultDto {
    let tools_config = routes::load_tools();
    let tool = match tools_config.tools.iter().find(|t| t.id == tool_id) {
        Some(t) => t,
        None => {
            return SwitchResultDto {
                success: false,
                tool_name: tool_id.clone(),
                target_url,
                backup_path: None,
                replaced_count: 0,
                base_url_found: false,
                error: Some(format!("未找到工具: {}", tool_id)),
            };
        }
    };

    let config_path = match config_paths::get_tool_config_path(
        &tool.config_dir,
        &tool.config_file,
    ) {
        Some(p) => p,
        None => {
            return SwitchResultDto {
                success: false,
                tool_name: tool.name.clone(),
                target_url,
                backup_path: None,
                replaced_count: 0,
                base_url_found: false,
                error: Some("无法确定配置文件路径".to_string()),
            };
        }
    };

    if !config_path.exists() {
        return SwitchResultDto {
            success: false,
            tool_name: tool.name.clone(),
            target_url,
            backup_path: None,
            replaced_count: 0,
            base_url_found: false,
            error: Some("配置文件不存在，可能未安装".to_string()),
        };
    }

    let format = match parsers::ConfigFormat::from_str(&tool.format) {
        Some(f) => f,
        None => {
            return SwitchResultDto {
                success: false,
                tool_name: tool.name.clone(),
                target_url,
                backup_path: None,
                replaced_count: 0,
                base_url_found: false,
                error: Some(format!("未知配置格式: {}", tool.format)),
            };
        }
    };

    let result = writer::safe_write(&config_path, format, &target_url, &tool.name);

    SwitchResultDto {
        success: result.success,
        tool_name: result.tool_name,
        target_url: result.target_url,
        backup_path: result.backup_path.map(|p| p.display().to_string()),
        replaced_count: result.replaced_count,
        base_url_found: result.base_url_found,
        error: result.error,
    }
}

/// 添加自定义线路
#[tauri::command]
#[allow(non_snake_case)]
fn add_custom_route(toolId: String, name: String, url: String) -> Result<(), String> {
    let mut config = routes::load_user_config();
    routes::add_custom_route(&mut config, toolId, name, url)
}

/// 编辑自定义线路
#[tauri::command]
fn edit_custom_route(index: usize, name: String, url: String) -> Result<(), String> {
    let mut config = routes::load_user_config();
    routes::edit_custom_route(&mut config, index, name, url)
}

/// 删除自定义线路
#[tauri::command]
fn delete_custom_route(index: usize) -> Result<(), String> {
    let mut config = routes::load_user_config();
    routes::delete_custom_route(&mut config, index)
}

/// 打开配置目录
#[tauri::command]
fn open_config_dir(config_dir: String) -> Result<(), String> {
    let dir = config_paths::get_tool_config_dir(&config_dir)
        .ok_or("无法确定配置目录路径")?;

    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    open_path_cross_platform(&dir)
}

/// 打开配置文件
#[tauri::command]
fn open_config_file(config_dir: String, config_file: String) -> Result<(), String> {
    let path = config_paths::get_tool_config_path(&config_dir, &config_file)
        .ok_or("无法确定配置文件路径")?;

    if !path.exists() {
        return Err("配置文件不存在".to_string());
    }

    open_path_cross_platform(&path)
}

/// 安装工具
#[tauri::command]
fn install_tool(tool_id: String, target_url: String) -> SwitchResultDto {
    let tools_config = routes::load_tools();
    let tool = match tools_config.tools.iter().find(|t| t.id == tool_id) {
        Some(t) => t,
        None => {
            return SwitchResultDto {
                success: false,
                tool_name: tool_id.clone(),
                target_url,
                backup_path: None,
                replaced_count: 0,
                base_url_found: false,
                error: Some(format!("未找到工具: {}", tool_id)),
            };
        }
    };

    let install_cmd = if cfg!(target_os = "windows") {
        &tool.install_commands.windows
    } else {
        &tool.install_commands.unix
    };

    let result = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", install_cmd])
            .status()
    } else {
        Command::new("sh")
            .args(["-c", install_cmd])
            .status()
    };

    match result {
        Ok(status) if status.success() => {
            let config_path = match config_paths::get_tool_config_path(
                &tool.config_dir,
                &tool.config_file,
            ) {
                Some(p) => p,
                None => {
                    return SwitchResultDto {
                        success: false,
                        tool_name: tool.name.clone(),
                        target_url,
                        backup_path: None,
                        replaced_count: 0,
                        base_url_found: false,
                        error: Some("无法确定配置文件路径".to_string()),
                    };
                }
            };

            let format = match parsers::ConfigFormat::from_str(&tool.format) {
                Some(f) => f,
                None => {
                    return SwitchResultDto {
                        success: false,
                        tool_name: tool.name.clone(),
                        target_url,
                        backup_path: None,
                        replaced_count: 0,
                        base_url_found: false,
                        error: Some(format!("未知配置格式: {}", tool.format)),
                    };
                }
            };

            let write_result = writer::write_default_config(
                &config_path,
                &tool.default_config,
                format,
                &target_url,
                &tool.name,
            );

            SwitchResultDto {
                success: write_result.success,
                tool_name: write_result.tool_name,
                target_url: write_result.target_url,
                backup_path: write_result.backup_path.map(|p| p.display().to_string()),
                replaced_count: write_result.replaced_count,
                base_url_found: write_result.base_url_found,
                error: write_result.error,
            }
        }
        Ok(_) => SwitchResultDto {
            success: false,
            tool_name: tool.name.clone(),
            target_url,
            backup_path: None,
            replaced_count: 0,
            base_url_found: false,
            error: Some(format!(
                "安装失败，请手动执行: {}",
                install_cmd
            )),
        },
        Err(e) => SwitchResultDto {
            success: false,
            tool_name: tool.name.clone(),
            target_url,
            backup_path: None,
            replaced_count: 0,
            base_url_found: false,
            error: Some(format!("执行安装命令失败: {} (命令: {})", e, install_cmd)),
        },
    }
}

/// 跨平台打开路径
fn open_path_cross_platform(path: &std::path::Path) -> Result<(), String> {
    let result = if cfg!(target_os = "windows") {
        // Windows: explorer 命令即使成功打开也可能返回非零退出码
        // 只要命令能执行（Ok），就认为成功
        Command::new("explorer").arg(path).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()
    } else {
        Command::new("xdg-open").arg(path).status()
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("无法打开: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tool_statuses,
            get_user_config,
            switch_route,
            add_custom_route,
            edit_custom_route,
            delete_custom_route,
            open_config_dir,
            open_config_file,
            install_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
