use crate::backup;
use crate::parsers::{ConfigFormat, ParsedConfig};
use std::path::{Path, PathBuf};

/// 切换结果
pub struct SwitchResult {
    pub tool_name: String,
    pub target_url: String,
    pub success: bool,
    pub backup_path: Option<PathBuf>,
    pub replaced_count: usize,
    pub base_url_found: bool,
    pub error: Option<String>,
}

/// 读取配置文件并查找 base_url
pub fn read_base_url(
    file_path: &Path,
    format: ConfigFormat,
) -> Result<Option<String>, String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 对于 TOML 和 ENV，使用正则表达式提取 URL，避免解析-序列化破坏格式
    match format {
        ConfigFormat::Toml => Ok(extract_base_url_toml(&content)),
        ConfigFormat::Env => Ok(extract_base_url_env(&content)),
        ConfigFormat::Json => {
            let config = ParsedConfig::parse(&content, format)
                .map_err(|e| format!("解析配置文件失败: {}", e))?;
            let urls = config.find_base_urls();
            Ok(urls.into_iter().next())
        }
    }
}

/// 使用正则表达式从 TOML 中提取 base_url
fn extract_base_url_toml(content: &str) -> Option<String> {
    // 匹配 base_url = "..." 或 base_url = '...'
    let re = regex::Regex::new(r#"(?i)base_url\s*=\s*["']([^"']+)["']"#).unwrap();
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// 使用正则表达式从 ENV 中提取 base_url
fn extract_base_url_env(content: &str) -> Option<String> {
    // 匹配包含 BASE_URL 的行，如 GOOGLE_GEMINI_BASE_URL=https://...
    // 使用多行模式 (?m) 使 ^ 和 $ 匹配每行的开头和结尾
    let re = regex::Regex::new(r#"(?im)^\s*\w*BASE_URL\w*\s*=\s*["']?([^"'\n]+)["']?\s*$"#).unwrap();
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// 安全写入：备份 → 解析 → 替换 → 序列化 → 写入 → 失败回滚
pub fn safe_write(
    file_path: &Path,
    format: ConfigFormat,
    new_url: &str,
    tool_name: &str,
) -> SwitchResult {
    let mut result = SwitchResult {
        tool_name: tool_name.to_string(),
        target_url: new_url.to_string(),
        success: false,
        backup_path: None,
        replaced_count: 0,
        base_url_found: false,
        error: None,
    };

    // 1. 读取原文件
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            result.error = Some(format!("读取配置文件失败: {}", e));
            return result;
        }
    };

    // 2. 根据格式选择替换方式
    let new_content = match format {
        ConfigFormat::Toml => {
            // 使用正则表达式替换，保留文件格式
            match replace_base_url_toml_regex(&content, new_url) {
                Ok((new_content, count, found)) => {
                    result.base_url_found = found;
                    result.replaced_count = count;
                    new_content
                }
                Err(e) => {
                    result.error = Some(format!("替换 base_url 失败: {}", e));
                    return result;
                }
            }
        }
        ConfigFormat::Env => {
            // 使用正则表达式替换，保留文件格式
            match replace_base_url_env_regex(&content, new_url) {
                Ok((new_content, count, found)) => {
                    result.base_url_found = found;
                    result.replaced_count = count;
                    new_content
                }
                Err(e) => {
                    result.error = Some(format!("替换 base_url 失败: {}", e));
                    return result;
                }
            }
        }
        ConfigFormat::Json => {
            // JSON 使用解析-序列化方式
            let mut config = match ParsedConfig::parse(&content, format) {
                Ok(c) => c,
                Err(e) => {
                    result.error = Some(format!("解析配置文件失败: {}", e));
                    return result;
                }
            };

            let existing_urls = config.find_base_urls();
            result.base_url_found = !existing_urls.is_empty();
            result.replaced_count = config.replace_base_url(new_url);

            match config.serialize() {
                Ok(s) => s,
                Err(e) => {
                    result.error = Some(format!("序列化配置文件失败: {}", e));
                    return result;
                }
            }
        }
    };

    // 3. 创建备份
    let backup_path = match backup::create_backup(file_path) {
        Ok(p) => p,
        Err(e) => {
            result.error = Some(format!("创建备份失败: {}", e));
            return result;
        }
    };
    result.backup_path = Some(backup_path.clone());

    // 4. 写入新内容
    match std::fs::write(file_path, new_content) {
        Ok(_) => {
            result.success = true;

            // 恢复文件权限（非 Windows）
            #[cfg(unix)]
            {
                if let Ok(original_metadata) = std::fs::metadata(&backup_path) {
                    let _ = std::fs::set_permissions(file_path, original_metadata.permissions());
                }
            }

            result
        }
        Err(e) => {
            // 5. 写入失败，回滚
            let rollback_error = backup::rollback(&backup_path, file_path)
                .err()
                .map(|e| format!("回滚也失败: {}", e));

            result.error = Some(format!(
                "写入配置文件失败: {}{}",
                e,
                rollback_error.map(|re| format!("（{}）", re)).unwrap_or_default()
            ));
            result
        }
    }
}

/// 使用正则表达式替换 TOML 中的 base_url，保留文件格式
fn replace_base_url_toml_regex(content: &str, new_url: &str) -> Result<(String, usize, bool), String> {
    let re = regex::Regex::new(r#"(?i)(base_url\s*=\s*["'])([^"']+)(["'])"#)
        .map_err(|e| format!("正则表达式错误: {}", e))?;
    
    let mut count = 0;
    let mut found = false;
    
    let new_content = re.replace_all(content, |caps: &regex::Captures| {
        found = true;
        count += 1;
        let quote = &caps[3]; // 保留原来的引号类型
        format!("{}{}{}", &caps[1], new_url, quote)
    });
    
    Ok((new_content.to_string(), count, found))
}

/// 使用正则表达式替换 ENV 中的 base_url，保留文件格式
fn replace_base_url_env_regex(content: &str, new_url: &str) -> Result<(String, usize, bool), String> {
    // 匹配 BASE_URL=value 或 BASE_URL="value"，保留前面的键名和等号格式
    // (?im) = case-insensitive + multiline（^ 和 $ 匹配每行）
    let re = regex::Regex::new(r#"(?im)^(\s*\w*BASE_URL\w*\s*=\s*["']?)([^"'\n]+)(["']?\s*)$"#)
        .map_err(|e| format!("正则表达式错误: {}", e))?;
    
    let mut count = 0;
    let mut found = false;
    
    let new_content = re.replace_all(content, |caps: &regex::Captures| {
        found = true;
        count += 1;
        let trailing = &caps[3]; // 保留行尾的引号和空格
        format!("{}{}{}", &caps[1], new_url, trailing)
    });
    
    Ok((new_content.to_string(), count, found))
}

/// 写入默认配置（安装后初始化）
pub fn write_default_config(
    file_path: &Path,
    default_config: &str,
    format: ConfigFormat,
    new_url: &str,
    tool_name: &str,
) -> SwitchResult {
    let mut result = SwitchResult {
        tool_name: tool_name.to_string(),
        target_url: new_url.to_string(),
        success: false,
        backup_path: None,
        replaced_count: 0,
        base_url_found: false,
        error: None,
    };

    // 如果配置文件已存在，先备份
    if file_path.exists() {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                result.error = Some(format!("读取已有配置文件失败: {}", e));
                return result;
            }
        };

        let backup_path = match backup::create_backup(file_path) {
            Ok(p) => p,
            Err(e) => {
                result.error = Some(format!("创建备份失败: {}", e));
                return result;
            }
        };
        result.backup_path = Some(backup_path);

        // 根据格式选择替换方式（与 safe_write 保持一致）
        let replace_result = match format {
            ConfigFormat::Toml => replace_base_url_toml_regex(&content, new_url),
            ConfigFormat::Env => replace_base_url_env_regex(&content, new_url),
            ConfigFormat::Json => {
                // JSON 使用解析-序列化方式
                match ParsedConfig::parse(&content, format) {
                    Ok(mut config) => {
                        let urls = config.find_base_urls();
                        let found = !urls.is_empty();
                        let count = config.replace_base_url(new_url);
                        match config.serialize() {
                            Ok(s) => Ok((s, count, found)),
                            Err(e) => Err(format!("序列化配置文件失败: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("解析配置文件失败: {}", e)),
                }
            }
        };

        match replace_result {
            Ok((new_content, count, found)) => {
                result.base_url_found = found;
                result.replaced_count = count;
                if found {
                    // 找到 base_url，写入替换后的内容
                    match std::fs::write(file_path, new_content) {
                        Ok(_) => {
                            result.success = true;
                            return result;
                        }
                        Err(e) => {
                            let _ = backup::rollback(
                                result.backup_path.as_ref().unwrap(),
                                file_path,
                            );
                            result.error = Some(format!("写入配置文件失败: {}", e));
                            return result;
                        }
                    }
                }
                // 未找到 base_url，继续用默认配置覆盖
            }
            Err(_e) => {
                // 替换失败，继续用默认配置覆盖
                result.base_url_found = false;
            }
        }
    }

    // 用默认配置初始化
    let mut config = match ParsedConfig::parse(default_config, format) {
        Ok(c) => c,
        Err(e) => {
            result.error = Some(format!("解析默认配置模板失败: {}", e));
            return result;
        }
    };

    let urls = config.find_base_urls();
    result.base_url_found = !urls.is_empty();
    result.replaced_count = config.replace_base_url(new_url);

    let new_content = match config.serialize() {
        Ok(s) => s,
        Err(e) => {
            result.error = Some(format!("序列化默认配置失败: {}", e));
            return result;
        }
    };

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                result.error = Some(format!("创建配置目录失败: {}", e));
                return result;
            }
        }
    }

    match std::fs::write(file_path, new_content) {
        Ok(_) => {
            result.success = true;
            result
        }
        Err(e) => {
            result.error = Some(format!("写入默认配置失败: {}", e));
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_url_env() {
        let content = r#"GEMINI_API_KEY=sk-xxx
GEMINI_MODEL=gemini-3.5-flash
GOOGLE_GEMINI_BASE_URL=https://api.claudecode.net.cn/api/gemini"#;

        let result = extract_base_url_env(content);
        assert!(result.is_some(), "应该能提取到 URL");
        assert_eq!(result.unwrap(), "https://api.claudecode.net.cn/api/gemini");
    }

    #[test]
    fn test_extract_base_url_env_with_quotes() {
        let content = r#"GOOGLE_GEMINI_BASE_URL="https://api.example.com/gemini""#;

        let result = extract_base_url_env(content);
        assert!(result.is_some(), "应该能提取到带引号的 URL");
        assert_eq!(result.unwrap(), "https://api.example.com/gemini");
    }
}
