#[cfg(test)]
mod tests {
    use crate::parsers::{ConfigFormat, ParsedConfig};

    #[test]
    fn test_env_simple_base_url() {
        let content = "GOOGLE_GEMINI_BASE_URL=https://api.example.com\n";
        let config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec!["https://api.example.com"]);
    }

    #[test]
    fn test_env_case_insensitive() {
        let content = "google_gemini_base_url=https://api.example.com\n";
        let config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls.len(), 1);
    }

    #[test]
    fn test_env_preserves_comments() {
        let content = "# This is a comment\nGOOGLE_GEMINI_BASE_URL=https://old.com\nAPI_KEY=xxx\n";
        let mut config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        config.replace_base_url("https://new.com");
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("# This is a comment"));
        assert!(serialized.contains("API_KEY=xxx"));
        assert!(serialized.contains("https://new.com"));
        assert!(!serialized.contains("https://old.com"));
    }

    #[test]
    fn test_env_preserves_blank_lines() {
        let content = "# Comment\n\nGOOGLE_GEMINI_BASE_URL=https://api.example.com\n";
        let config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("\n\n"));
    }

    #[test]
    fn test_env_replace() {
        let content = "GOOGLE_GEMINI_BASE_URL=https://old.com\nAPI_KEY=xxx\n";
        let mut config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let count = config.replace_base_url("https://new.com");
        assert_eq!(count, 1);
        let serialized = config.serialize().unwrap();
        assert!(serialized.contains("GOOGLE_GEMINI_BASE_URL=https://new.com"));
    }

    #[test]
    fn test_env_empty_value() {
        let content = "GOOGLE_GEMINI_BASE_URL=\n";
        let config = ParsedConfig::parse(content, ConfigFormat::Env).unwrap();
        let urls = config.find_base_urls();
        assert_eq!(urls, vec![""]);
    }
}
