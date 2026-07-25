pub mod env;
pub mod json;
pub mod toml;

use std::fmt;

/// 配置文件格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Env,
}

impl ConfigFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "env" => Some(Self::Env),
            _ => None,
        }
    }
}

impl fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Toml => write!(f, "TOML"),
            Self::Env => write!(f, "ENV"),
        }
    }
}

/// 解析错误
#[derive(Debug)]
pub enum ParseError {
    Json(serde_json::Error),
    Toml(::toml::de::Error),
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "JSON 解析错误: {}", e),
            Self::Toml(e) => write!(f, "TOML 解析错误: {}", e),
            Self::Io(e) => write!(f, "IO 错误: {}", e),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<::toml::de::Error> for ParseError {
    fn from(e: ::toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// ENV 文件中的一行
#[derive(Debug, Clone)]
pub struct EnvLine {
    /// 键名（如果是键值对行）
    pub key: String,
    /// 值（如果是键值对行）
    pub value: String,
    /// 是否为注释或空行
    pub is_comment: bool,
    /// 原始行内容
    pub raw: String,
}

/// 解析后的配置（枚举持有不同格式的解析结果）
pub enum ParsedConfig {
    Json(serde_json::Value),
    Toml(::toml::Value),
    Env(Vec<EnvLine>),
}

impl ParsedConfig {
    /// 根据格式解析配置内容
    pub fn parse(content: &str, format: ConfigFormat) -> Result<Self, ParseError> {
        match format {
            ConfigFormat::Json => {
                let value: serde_json::Value = serde_json::from_str(content)?;
                Ok(Self::Json(value))
            }
            ConfigFormat::Toml => {
                let value: ::toml::Value = ::toml::from_str(content)?;
                Ok(Self::Toml(value))
            }
            ConfigFormat::Env => Ok(Self::Env(parse_env(content))),
        }
    }

    /// 递归查找所有包含 base_url 的字段值
    pub fn find_base_urls(&self) -> Vec<String> {
        match self {
            Self::Json(v) => find_base_urls_json(v),
            Self::Toml(v) => find_base_urls_toml(v),
            Self::Env(lines) => lines
                .iter()
                .filter(|l| !l.is_comment && l.key.to_lowercase().contains("base_url"))
                .map(|l| l.value.clone())
                .collect(),
        }
    }

    /// 替换所有 base_url 字段的值，返回替换数量
    pub fn replace_base_url(&mut self, new_url: &str) -> usize {
        match self {
            Self::Json(v) => replace_base_url_json(v, new_url),
            Self::Toml(v) => replace_base_url_toml(v, new_url),
            Self::Env(lines) => {
                let mut count = 0;
                for line in lines.iter_mut() {
                    if !line.is_comment && line.key.to_lowercase().contains("base_url") {
                        line.value = new_url.to_string();
                        line.raw = format!("{}={}", line.key, new_url);
                        count += 1;
                    }
                }
                count
            }
        }
    }

    /// 序列化回字符串
    pub fn serialize(&self) -> Result<String, ParseError> {
        match self {
            Self::Json(v) => {
                let s = serde_json::to_string_pretty(v)?;
                Ok(s)
            }
            Self::Toml(v) => {
                let s = ::toml::to_string(v).map_err(|e| ParseError::Other(e.to_string()))?;
                Ok(s)
            }
            Self::Env(lines) => {
                let mut result = String::new();
                for line in lines {
                    result.push_str(&line.raw);
                    result.push('\n');
                }
                Ok(result)
            }
        }
    }
}

/// 解析 ENV 格式内容
fn parse_env(content: &str) -> Vec<EnvLine> {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                EnvLine {
                    key: String::new(),
                    value: String::new(),
                    is_comment: true,
                    raw: line.to_string(),
                }
            } else if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().to_string();
                EnvLine {
                    key,
                    value,
                    is_comment: false,
                    raw: line.to_string(),
                }
            } else {
                EnvLine {
                    key: String::new(),
                    value: String::new(),
                    is_comment: true,
                    raw: line.to_string(),
                }
            }
        })
        .collect()
}

/// 递归查找 JSON 中的 base_url 字段值
fn find_base_urls_json(value: &serde_json::Value) -> Vec<String> {
    let mut results = Vec::new();
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if k.to_lowercase().contains("base_url") {
                    if let Some(s) = v.as_str() {
                        results.push(s.to_string());
                    } else {
                        results.push(v.to_string());
                    }
                }
                results.extend(find_base_urls_json(v));
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                results.extend(find_base_urls_json(item));
            }
        }
        _ => {}
    }
    results
}

/// 递归替换 JSON 中的 base_url 字段值
fn replace_base_url_json(value: &mut serde_json::Value, new_url: &str) -> usize {
    let mut count = 0;
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if k.to_lowercase().contains("base_url") {
                    *v = serde_json::Value::String(new_url.to_string());
                    count += 1;
                }
                count += replace_base_url_json(v, new_url);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                count += replace_base_url_json(item, new_url);
            }
        }
        _ => {}
    }
    count
}

/// 递归查找 TOML 中的 base_url 字段值
fn find_base_urls_toml(value: &::toml::Value) -> Vec<String> {
    let mut results = Vec::new();
    match value {
        ::toml::Value::Table(table) => {
            for (k, v) in table {
                if k.to_lowercase().contains("base_url") {
                    // 提取字符串值，不添加额外引号
                    if let Some(s) = v.as_str() {
                        results.push(s.to_string());
                    } else {
                        results.push(v.to_string());
                    }
                }
                results.extend(find_base_urls_toml(v));
            }
        }
        ::toml::Value::Array(arr) => {
            for item in arr {
                results.extend(find_base_urls_toml(item));
            }
        }
        _ => {}
    }
    results
}

/// 递归替换 TOML 中的 base_url 字段值
fn replace_base_url_toml(value: &mut ::toml::Value, new_url: &str) -> usize {
    let mut count = 0;
    match value {
        ::toml::Value::Table(table) => {
            for (k, v) in table.iter_mut() {
                if k.to_lowercase().contains("base_url") {
                    *v = ::toml::Value::String(new_url.to_string());
                    count += 1;
                }
                count += replace_base_url_toml(v, new_url);
            }
        }
        ::toml::Value::Array(arr) => {
            for item in arr.iter_mut() {
                count += replace_base_url_toml(item, new_url);
            }
        }
        _ => {}
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let content = r#"{"ANTHROPIC_BASE_URL": "https://old.example.com", "key": "xxx"}"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec!["https://old.example.com"]);
    }

    #[test]
    fn test_replace_json() {
        let content = r#"{"ANTHROPIC_BASE_URL": "https://old.example.com"}"#;
        let mut config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let count = config.replace_base_url("https://new.example.com");
        assert_eq!(count, 1);
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("https://new.example.com"));
    }

    #[test]
    fn test_parse_toml() {
        let content = r#"model = "o4"
base_url = "https://old.example.com"
"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://old.example.com");
    }

    #[test]
    fn test_replace_toml() {
        let content = r#"base_url = "https://old.example.com"
"#;
        let mut config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        let count = config.replace_base_url("https://new.example.com");
        assert_eq!(count, 1);
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("https://new.example.com"));
    }

    #[test]
    fn test_parse_env() {
        let content = "# Comment\nGOOGLE_GEMINI_BASE_URL=https://old.example.com\nAPI_KEY=xxx\n";
        let config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec!["https://old.example.com"]);
    }

    #[test]
    fn test_replace_env() {
        let content = "# Comment\nGOOGLE_GEMINI_BASE_URL=https://old.example.com\nAPI_KEY=xxx\n";
        let mut config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let count = config.replace_base_url("https://new.example.com");
        assert_eq!(count, 1);
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("https://new.example.com"));
        assert!(serialized.contains("# Comment"));
        assert!(serialized.contains("API_KEY=xxx"));
    }

    #[test]
    fn test_nested_json_base_url() {
        let content = r#"{"section": {"base_url": "https://nested.example.com"}}"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec!["https://nested.example.com"]);
    }

    #[test]
    fn test_multiple_base_urls_json() {
        let content = r#"{"base_url": "https://a.com", "nested": {"BASE_URL": "https://b.com"}}"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 2);
    }
}
