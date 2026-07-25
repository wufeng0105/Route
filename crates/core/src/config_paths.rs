use dirs::home_dir;
use std::path::PathBuf;

/// 获取用户 Home 目录
#[allow(dead_code)]
pub fn get_home_dir() -> Option<PathBuf> {
    home_dir()
}

/// 获取工具配置文件路径 (~/<config_dir>/<config_file>)
pub fn get_tool_config_path(config_dir: &str, config_file: &str) -> Option<PathBuf> {
    home_dir().map(|h| h.join(config_dir).join(config_file))
}

/// 获取工具配置目录路径 (~/<config_dir>)
pub fn get_tool_config_dir(config_dir: &str) -> Option<PathBuf> {
    home_dir().map(|h| h.join(config_dir))
}

/// 获取用户配置目录路径 (~/.api-line-switcher/)
pub fn get_user_config_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".api-line-switcher"))
}

/// 获取用户配置文件路径 (~/.api-line-switcher/config.json)
pub fn get_user_config_path() -> Option<PathBuf> {
    get_user_config_dir().map(|d| d.join("config.json"))
}

/// 确保用户配置目录存在，返回目录路径
pub fn ensure_user_config_dir() -> std::io::Result<PathBuf> {
    let config_dir = get_user_config_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "无法确定 Home 目录",
        )
    })?;
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}

/// 检查工具配置文件是否存在
pub fn tool_config_exists(config_dir: &str, config_file: &str) -> bool {
    get_tool_config_path(config_dir, config_file)
        .map(|p| p.exists())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_home_dir() {
        let home = get_home_dir();
        assert!(home.is_some(), "Home directory should be detected");
    }

    #[test]
    fn test_get_tool_config_path() {
        let path = get_tool_config_path(".codex", "config.toml");
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(".codex"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_get_user_config_path() {
        let path = get_user_config_path();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(".api-line-switcher"));
        assert!(path.to_string_lossy().contains("config.json"));
    }
}
