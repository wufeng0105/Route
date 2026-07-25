#[cfg(test)]
mod tests {
    use crate::parsers::{ConfigFormat, ParsedConfig};

    #[test]
    fn test_json_simple_base_url() {
        let content = r#"{"ANTHROPIC_BASE_URL": "https://api.example.com"}"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec!["https://api.example.com"]);
    }

    #[test]
    fn test_json_nested_base_url() {
        let content = r#"{
            "section": {
                "base_url": "https://nested.example.com"
            }
        }"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec!["https://nested.example.com"]);
    }

    #[test]
    fn test_json_case_insensitive() {
        let content = r#"{"Base_Url": "https://api.example.com"}"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_json_replace_and_serialize() {
        let content = r#"{"ANTHROPIC_BASE_URL": "https://old.com"}"#;
        let mut config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let count = config.replace_base_url("https://new.com");
        assert_eq!(count, 1);
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("https://new.com"));
        assert!(!serialized.contains("https://old.com"));
    }

    #[test]
    fn test_json_preserves_other_fields() {
        let content = r#"{"ANTHROPIC_BASE_URL": "https://old.com", "api_key": "sk-xxx"}"#;
        let mut config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        config.replace_base_url("https://new.com");
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("sk-xxx"));
    }

    #[test]
    fn test_json_array_base_url() {
        let content = r#"{"endpoints": [{"base_url": "https://a.com"}, {"base_url": "https://b.com"}]}"#;
        let config = ParsedConfig::parse(content, ConfigFormat::Json).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 2);
    }
}
