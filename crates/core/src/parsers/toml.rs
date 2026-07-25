#[cfg(test)]
mod tests {
    use crate::parsers::{ConfigFormat, ParsedConfig};

    #[test]
    fn test_toml_simple_base_url() {
        let content = r#"model = "o4"
base_url = "https://api.example.com"
"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_toml_nested_base_url() {
        let content = r#"[api]
base_url = "https://nested.example.com"
"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_toml_case_insensitive() {
        let content = r#"Base_URL = "https://api.example.com"
"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_toml_replace_and_serialize() {
        let content = r#"model = "o4"
base_url = "https://old.com"
"#;
        let mut config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        let count = config.replace_base_url("https://new.com");
        assert_eq!(count, 1);
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("https://new.com"));
        assert!(!serialized.contains("https://old.com"));
    }

    #[test]
    fn test_toml_preserves_other_fields() {
        let content = r#"model = "o4"
base_url = "https://old.com"
api_key = "sk-xxx"
"#;
        let mut config = ParsedConfig::parse(content, ConfigFormat::Toml).unwrap();
        config.replace_base_url("https://new.com");
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("o4"));
        assert!(serialized.contains("sk-xxx"));
    }
}
