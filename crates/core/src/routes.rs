use crate::config_paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具定义（来自 tools.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub id: String,
    pub name: String,
    #[serde(rename = "configDir")]
    pub config_dir: String,
    #[serde(rename = "configFile")]
    pub config_file: String,
    pub format: String,
    #[serde(rename = "installCommands")]
    pub install_commands: InstallCommands,
    #[serde(rename = "defaultConfig")]
    pub default_config: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallCommands {
    pub windows: String,
    pub unix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub tools: Vec<ToolDef>,
}

/// 预设线路
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetRoute {
    pub id: String,
    pub name: String,
    pub urls: HashMap<String, String>,
}

/// 自定义线路（每个工具独立）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRoute {
    #[serde(rename = "toolId")]
    pub tool_id: String,
    pub name: String,
    pub url: String,
}

/// 用户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(rename = "presetRoutes")]
    pub preset_routes: Vec<PresetRoute>,
    #[serde(rename = "customRoutes")]
    pub custom_routes: Vec<CustomRoute>,
}

/// 加载工具定义（从嵌入的 tools.json）
pub fn load_tools() -> ToolsConfig {
    let json = include_str!("tools.json");
    serde_json::from_str(json).expect("tools.json 解析失败")
}

/// 创建默认用户配置
pub fn default_user_config() -> UserConfig {
    let mut global_urls = HashMap::new();
    global_urls.insert(
        "codex".to_string(),
        "https://api.aicodemirror.ai/api/codex/backend-api/codex".to_string(),
    );
    global_urls.insert(
        "claude".to_string(),
        "https://api.aicodemirror.ai/api/claudecode".to_string(),
    );
    global_urls.insert(
        "gemini".to_string(),
        "https://api.aicodemirror.ai/api/gemini".to_string(),
    );

    let mut domestic_urls = HashMap::new();
    domestic_urls.insert(
        "codex".to_string(),
        "https://api.claudecode.net.cn/api/codex/backend-api/codex".to_string(),
    );
    domestic_urls.insert(
        "claude".to_string(),
        "https://api.claudecode.net.cn/api/claudecode".to_string(),
    );
    domestic_urls.insert(
        "gemini".to_string(),
        "https://api.claudecode.net.cn/api/gemini".to_string(),
    );

    UserConfig {
        preset_routes: vec![
            PresetRoute {
                id: "global".to_string(),
                name: "全球高保".to_string(),
                urls: global_urls,
            },
            PresetRoute {
                id: "domestic".to_string(),
                name: "国内优化".to_string(),
                urls: domestic_urls,
            },
        ],
        custom_routes: vec![],
    }
}

/// 加载用户配置
pub fn load_user_config() -> UserConfig {
    let config_path = match config_paths::get_user_config_path() {
        Some(p) => p,
        None => return default_user_config(),
    };

    if !config_path.exists() {
        return default_user_config();
    }

    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            // 尝试解析，如果失败则合并默认预设线路
            match serde_json::from_str::<UserConfig>(&content) {
                Ok(mut config) => {
                    // 确保预设线路始终存在且为最新
                    let default = default_user_config();
                    config.preset_routes = default.preset_routes;
                    config
                }
                Err(_) => {
                    // 解析失败，使用默认配置但保留尝试读取自定义线路
                    let mut config = default_user_config();
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(custom) = json.get("customRoutes") {
                            if let Ok(routes) =
                                serde_json::from_value::<Vec<CustomRoute>>(custom.clone())
                            {
                                config.custom_routes = routes;
                            }
                        }
                    }
                    config
                }
            }
        }
        Err(_) => default_user_config(),
    }
}

/// 保存用户配置（只保存自定义线路，预设线路始终从代码加载）
pub fn save_user_config(config: &UserConfig) -> Result<(), String> {
    let config_dir = config_paths::ensure_user_config_dir()
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let config_path = config_dir.join("config.json");

    // 只保存自定义线路，预设线路不写入用户文件
    let save_data = serde_json::json!({
        "customRoutes": config.custom_routes
    });

    let json = serde_json::to_string_pretty(&save_data)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(&config_path, json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 添加自定义线路（指定工具）
pub fn add_custom_route(
    config: &mut UserConfig,
    tool_id: String,
    name: String,
    url: String,
) -> Result<(), String> {
    // 校验
    if tool_id.trim().is_empty() {
        return Err("工具 ID 不能为空".to_string());
    }
    if name.trim().is_empty() {
        return Err("线路名称不能为空".to_string());
    }
    if !is_valid_url(&url) {
        return Err("URL 格式无效，需以 http:// 或 https:// 开头".to_string());
    }

    // 检查同工具下名称是否重复
    if config
        .custom_routes
        .iter()
        .any(|r| r.tool_id == tool_id && r.name == name)
    {
        return Err(format!("该工具下线路名称「{}」已存在", name));
    }

    config.custom_routes.push(CustomRoute {
        tool_id,
        name,
        url,
    });
    save_user_config(config)?;
    Ok(())
}

/// 编辑自定义线路
pub fn edit_custom_route(
    config: &mut UserConfig,
    index: usize,
    name: String,
    url: String,
) -> Result<(), String> {
    if index >= config.custom_routes.len() {
        return Err("线路索引无效".to_string());
    }
    if name.trim().is_empty() {
        return Err("线路名称不能为空".to_string());
    }
    if !is_valid_url(&url) {
        return Err("URL 格式无效，需以 http:// 或 https:// 开头".to_string());
    }

    let tool_id = config.custom_routes[index].tool_id.clone();

    // 检查同工具下名称是否与其他线路重复（排除自身）
    if config
        .custom_routes
        .iter()
        .enumerate()
        .any(|(i, r)| i != index && r.tool_id == tool_id && r.name == name)
    {
        return Err(format!("该工具下线路名称「{}」已存在", name));
    }

    config.custom_routes[index].name = name;
    config.custom_routes[index].url = url;
    save_user_config(config)?;
    Ok(())
}

/// 删除自定义线路
pub fn delete_custom_route(config: &mut UserConfig, index: usize) -> Result<(), String> {
    if index >= config.custom_routes.len() {
        return Err("线路索引无效".to_string());
    }

    config.custom_routes.remove(index);
    save_user_config(config)?;
    Ok(())
}

/// 校验 URL 格式
fn is_valid_url(url: &str) -> bool {
    (url.starts_with("http://") || url.starts_with("https://")) && url.len() > 8
}

/// 获取工具对应的线路 URL
#[allow(dead_code)]
pub fn get_route_url_for_tool<'a>(
    preset: &'a PresetRoute,
    tool_id: &str,
) -> Option<&'a str> {
    preset.urls.get(tool_id).map(|s| s.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tools() {
        let tools = load_tools();
        assert_eq!(tools.tools.len(), 3);
        assert_eq!(tools.tools[0].id, "codex");
        assert_eq!(tools.tools[1].id, "claude");
        assert_eq!(tools.tools[2].id, "gemini");
    }

    #[test]
    fn test_default_user_config() {
        let config = default_user_config();
        assert_eq!(config.preset_routes.len(), 2);
        assert_eq!(config.preset_routes[0].id, "global");
        assert_eq!(config.preset_routes[1].id, "domestic");
        assert!(config.custom_routes.is_empty());
    }

    #[test]
    fn test_is_valid_url() {
    assert!(is_valid_url("https://api.example.com"));
    assert!(is_valid_url("http://localhost:8080"));
    assert!(!is_valid_url("not-a-url"));
    assert!(!is_valid_url("ftp://example.com"));
    // 回归测试：运算符优先级 Bug 修复
    assert!(!is_valid_url("http://"), "http:// 本身不应判为合法 URL");
    assert!(!is_valid_url("https://"), "https:// 本身不应判为合法 URL");
    }

    #[test]
    fn test_get_route_url_for_tool() {
        let config = default_user_config();
        let global = &config.preset_routes[0];
        let url = get_route_url_for_tool(global, "codex");
        assert!(url.is_some());
        assert!(url.unwrap().contains("aicodemirror.ai"));
    }
}
