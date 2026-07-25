//! 集成测试：覆盖核心功能的端到端流程
//!
//! 测试范围：
//! - safe_write 对 TOML/JSON/ENV 三种格式的实际文件写入
//! - 备份创建与回滚流程
//! - write_default_config 新配置初始化
//! - 线路管理（增删改 + 校验）
//! - 路径解析

use route_core::backup;
use route_core::config_paths;
use route_core::parsers::{ConfigFormat, ParsedConfig};
use route_core::routes;
use route_core::writer;
use std::fs;

// ========== safe_write 集成测试 ==========

#[test]
fn test_safe_write_toml_end_to_end() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("integ_test_config.toml");
    let _ = fs::remove_file(&file_path);

    fs::write(
        &file_path,
        r#"model = "o4-mini"
base_url = "https://old.example.com/codex"
api_key = "sk-test123"
"#,
    )
    .unwrap();

    let result = writer::safe_write(
        &file_path,
        ConfigFormat::Toml,
        "https://new.example.com/codex",
        "Codex CLI",
    );

    assert!(result.success, "切换应成功");
    assert!(result.base_url_found, "应找到 base_url");
    assert_eq!(result.replaced_count, 1, "应替换 1 个 base_url");
    assert!(result.backup_path.is_some(), "应创建备份");

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("https://new.example.com/codex"));
    assert!(!content.contains("https://old.example.com/codex"));
    assert!(content.contains("o4-mini"));
    assert!(content.contains("sk-test123"));

    let backup = result.backup_path.unwrap();
    assert!(backup.exists());
    let backup_content = fs::read_to_string(&backup).unwrap();
    assert!(backup_content.contains("https://old.example.com/codex"));

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_file(&backup);
}

#[test]
fn test_safe_write_json_end_to_end() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("integ_test_settings.json");
    let _ = fs::remove_file(&file_path);

    fs::write(
        &file_path,
        r#"{
  "ANTHROPIC_BASE_URL": "https://old.example.com/claude",
  "api_key": "sk-ant-test456"
}"#,
    )
    .unwrap();

    let result = writer::safe_write(
        &file_path,
        ConfigFormat::Json,
        "https://new.example.com/claude",
        "Claude Code",
    );

    assert!(result.success);
    assert!(result.base_url_found);
    assert_eq!(result.replaced_count, 1);

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("https://new.example.com/claude"));
    assert!(!content.contains("https://old.example.com/claude"));
    assert!(content.contains("sk-ant-test456"));

    let _ = fs::remove_file(&file_path);
    if let Some(bp) = result.backup_path {
        let _ = fs::remove_file(&bp);
    }
}

#[test]
fn test_safe_write_env_end_to_end() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("integ_test_dotenv.env");
    let _ = fs::remove_file(&file_path);

    fs::write(
        &file_path,
        "# Gemini CLI config\nGOOGLE_GEMINI_BASE_URL=https://old.example.com/gemini\nAPI_KEY=sk-gem-test789\n",
    )
    .unwrap();

    let result = writer::safe_write(
        &file_path,
        ConfigFormat::Env,
        "https://new.example.com/gemini",
        "Gemini CLI",
    );

    assert!(result.success);
    assert!(result.base_url_found);
    assert_eq!(result.replaced_count, 1);

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("https://new.example.com/gemini"));
    assert!(!content.contains("https://old.example.com/gemini"));
    assert!(content.contains("# Gemini CLI config"));
    assert!(content.contains("API_KEY=sk-gem-test789"));

    let _ = fs::remove_file(&file_path);
    if let Some(bp) = result.backup_path {
        let _ = fs::remove_file(&bp);
    }
}

#[test]
fn test_safe_write_preserves_backup_content() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("integ_backup_preserve.toml");
    let _ = fs::remove_file(&file_path);

    let original_content = r#"model = "o4"
base_url = "https://original.example.com"
"#;
    fs::write(&file_path, original_content).unwrap();

    let result = writer::safe_write(
        &file_path,
        ConfigFormat::Toml,
        "https://changed.example.com",
        "TestTool",
    );

    assert!(result.success);
    let backup_path = result.backup_path.unwrap();
    let backup_content = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(
        backup_content, original_content,
        "备份内容应与原始内容一致"
    );

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_file(&backup_path);
}

// ========== 备份与回滚集成测试 ==========

#[test]
fn test_backup_and_rollback_full_flow() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("integ_rollback_test.json");
    let _ = fs::remove_file(&file_path);

    fs::write(&file_path, r#"{"base_url": "https://original.example.com"}"#).unwrap();

    let backup_path = backup::create_backup(&file_path).unwrap();
    assert!(backup_path.exists());

    // 模拟写入失败：破坏文件
    fs::write(&file_path, "CORRUPTED_CONTENT").unwrap();

    // 执行回滚
    backup::rollback(&backup_path, &file_path).unwrap();

    // 验证恢复
    let restored = fs::read_to_string(&file_path).unwrap();
    assert!(
        restored.contains("https://original.example.com"),
        "回滚后应恢复原始内容"
    );
    assert!(!backup_path.exists(), "回滚后备份文件应被重命名");

    let _ = fs::remove_file(&file_path);
}

#[test]
fn test_backup_timestamp_format() {
    let dir = std::env::temp_dir();
    let file_path = dir.join("integ_timestamp_test.txt");
    fs::write(&file_path, "test").unwrap();

    let backup_path = backup::create_backup(&file_path).unwrap();
    let backup_name = backup_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    assert!(
        backup_name.contains(".backup."),
        "备份文件名应包含 .backup. 标识"
    );

    let parts: Vec<&str> = backup_name.split(".backup.").collect();
    assert_eq!(parts.len(), 2);
    let timestamp = parts[1];
    assert_eq!(
        timestamp.len(),
        15,
        "时间戳应为 YYYYMMDD_HHMMSS 格式（15 字符）"
    );
    assert!(timestamp.contains('_'));

    let _ = fs::remove_file(&file_path);
    let _ = fs::remove_file(&backup_path);
}

// ========== write_default_config 集成测试 ==========

#[test]
fn test_write_default_config_new_file() {
    let dir = std::env::temp_dir().join("integ_default_config_new");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let file_path = dir.join("config.toml");
    let default_config = r#"model = "o4-mini"
base_url = ""
"#;

    let result = writer::write_default_config(
        &file_path,
        default_config,
        ConfigFormat::Toml,
        "https://preset.example.com",
        "Codex CLI",
    );

    assert!(result.success, "应成功写入默认配置");
    assert!(result.base_url_found, "默认配置中应有 base_url 字段");
    assert_eq!(result.replaced_count, 1);

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("https://preset.example.com"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_write_default_config_overwrite_existing() {
    let dir = std::env::temp_dir().join("integ_default_config_overwrite");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let file_path = dir.join("settings.json");
    fs::write(
        &file_path,
        r#"{"ANTHROPIC_BASE_URL": "https://old.example.com", "key": "val"}"#,
    )
    .unwrap();

    let default_config = r#"{"ANTHROPIC_BASE_URL": ""}"#;

    let result = writer::write_default_config(
        &file_path,
        default_config,
        ConfigFormat::Json,
        "https://new.example.com",
        "Claude Code",
    );

    assert!(result.success);
    assert!(result.base_url_found);
    assert!(result.backup_path.is_some(), "已有文件应创建备份");

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("https://new.example.com"));
    assert!(content.contains("val"), "原有其他字段应保留");

    let _ = fs::remove_dir_all(&dir);
}

// ========== 线路管理集成测试 ==========

#[test]
fn test_add_and_delete_custom_route() {
    let mut config = routes::default_user_config();
    assert!(config.custom_routes.is_empty());

    routes::add_custom_route(
        &mut config,
        "codex".to_string(),
        "测试线路A".to_string(),
        "https://test-a.example.com".to_string(),
    )
    .unwrap();

    assert_eq!(config.custom_routes.len(), 1);
    assert_eq!(config.custom_routes[0].name, "测试线路A");
    assert_eq!(config.custom_routes[0].tool_id, "codex");

    routes::add_custom_route(
        &mut config,
        "claude".to_string(),
        "测试线路B".to_string(),
        "https://test-b.example.com".to_string(),
    )
    .unwrap();

    assert_eq!(config.custom_routes.len(), 2);

    routes::delete_custom_route(&mut config, 0).unwrap();
    assert_eq!(config.custom_routes.len(), 1);
    assert_eq!(config.custom_routes[0].name, "测试线路B");

    if let Some(path) = config_paths::get_user_config_path() {
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn test_add_duplicate_route_name_rejected() {
    let mut config = routes::default_user_config();

    routes::add_custom_route(
        &mut config,
        "codex".to_string(),
        "重复线路".to_string(),
        "https://dup-1.example.com".to_string(),
    )
    .unwrap();

    let result = routes::add_custom_route(
        &mut config,
        "codex".to_string(),
        "重复线路".to_string(),
        "https://dup-2.example.com".to_string(),
    );

    assert!(result.is_err(), "同工具下同名线路应被拒绝");
    assert_eq!(config.custom_routes.len(), 1);

    if let Some(path) = config_paths::get_user_config_path() {
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn test_add_same_name_different_tool_allowed() {
    let mut config = routes::default_user_config();

    routes::add_custom_route(
        &mut config,
        "codex".to_string(),
        "共用名称".to_string(),
        "https://shared-codex.example.com".to_string(),
    )
    .unwrap();

    let result = routes::add_custom_route(
        &mut config,
        "claude".to_string(),
        "共用名称".to_string(),
        "https://shared-claude.example.com".to_string(),
    );

    assert!(result.is_ok(), "不同工具下同名线路应允许");
    assert_eq!(config.custom_routes.len(), 2);

    if let Some(path) = config_paths::get_user_config_path() {
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn test_edit_custom_route() {
    let mut config = routes::default_user_config();

    routes::add_custom_route(
        &mut config,
        "gemini".to_string(),
        "原始名称".to_string(),
        "https://original.example.com".to_string(),
    )
    .unwrap();

    routes::edit_custom_route(
        &mut config,
        0,
        "修改后名称".to_string(),
        "https://modified.example.com".to_string(),
    )
    .unwrap();

    assert_eq!(config.custom_routes[0].name, "修改后名称");
    assert_eq!(
        config.custom_routes[0].url,
        "https://modified.example.com"
    );

    if let Some(path) = config_paths::get_user_config_path() {
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn test_edit_route_index_out_of_bounds() {
    let mut config = routes::default_user_config();
    assert!(config.custom_routes.is_empty());

    let result = routes::edit_custom_route(
        &mut config,
        0,
        "test".to_string(),
        "https://test.example.com".to_string(),
    );

    assert!(result.is_err(), "空列表编辑应报错");
}

#[test]
fn test_delete_route_index_out_of_bounds() {
    let mut config = routes::default_user_config();
    let result = routes::delete_custom_route(&mut config, 99);
    assert!(result.is_err(), "越界删除应报错");
}

#[test]
fn test_invalid_url_rejected() {
    let mut config = routes::default_user_config();

    let result = routes::add_custom_route(
        &mut config,
        "codex".to_string(),
        "bad".to_string(),
        "ftp://invalid.example.com".to_string(),
    );

    assert!(result.is_err(), "非 http/https URL 应被拒绝");

    if let Some(path) = config_paths::get_user_config_path() {
        let _ = fs::remove_file(&path);
    }
}

// ========== 路径解析集成测试 ==========

#[test]
fn test_config_path_contains_home() {
    let path = config_paths::get_tool_config_path(".codex", "config.toml");
    assert!(path.is_some());

    let path_str = path.unwrap().to_string_lossy().to_string();
    assert!(path_str.contains(".codex"));
    assert!(path_str.contains("config.toml"));
}

#[test]
fn test_all_tool_paths_resolve() {
    let tools = routes::load_tools();
    for tool in &tools.tools {
        let path = config_paths::get_tool_config_path(&tool.config_dir, &tool.config_file);
        assert!(path.is_some(), "工具 {} 的路径应能解析", tool.id);
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains(&tool.config_dir));
        assert!(p.to_string_lossy().contains(&tool.config_file));
    }
}

// ========== 解析器端到端测试 ==========

#[test]
fn test_toml_full_round_trip() {
    let original = r#"model = "o4-mini"
base_url = "https://api.example.com"
api_key = "sk-xxx"
"#;
    let config = ParsedConfig::parse(original, ConfigFormat::Toml).unwrap();
    let urls = config.find_base_urls();
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://api.example.com");

    let mut config = config;
    let count = config.replace_base_url("https://replaced.example.com");
    assert_eq!(count, 1);

    let serialized = config.serialize().unwrap();
    let reparsed = ParsedConfig::parse(&serialized, ConfigFormat::Toml).unwrap();
    let new_urls = reparsed.find_base_urls();
    assert_eq!(new_urls[0], "https://replaced.example.com");
}

#[test]
fn test_json_full_round_trip() {
    let original = r#"{"ANTHROPIC_BASE_URL": "https://api.example.com", "key": "val"}"#;
    let config = ParsedConfig::parse(original, ConfigFormat::Json).unwrap();
    let urls = config.find_base_urls();
    assert_eq!(urls.len(), 1);

    let mut config = config;
    config.replace_base_url("https://replaced.example.com");

    let serialized = config.serialize().unwrap();
    let reparsed = ParsedConfig::parse(&serialized, ConfigFormat::Json).unwrap();
    let new_urls = reparsed.find_base_urls();
    assert_eq!(new_urls[0], "https://replaced.example.com");
    assert!(serialized.contains("val"));
}

#[test]
fn test_env_full_round_trip() {
    let original = "# Comment\nGOOGLE_GEMINI_BASE_URL=https://api.example.com\nAPI_KEY=xxx\n";
    let config = ParsedConfig::parse(original, ConfigFormat::Env).unwrap();
    let urls = config.find_base_urls();
    assert_eq!(urls.len(), 1);

    let mut config = config;
    config.replace_base_url("https://replaced.example.com");

    let serialized = config.serialize().unwrap();
    let reparsed = ParsedConfig::parse(&serialized, ConfigFormat::Env).unwrap();
    let new_urls = reparsed.find_base_urls();
    assert_eq!(new_urls[0], "https://replaced.example.com");
    assert!(serialized.contains("# Comment"));
    assert!(serialized.contains("API_KEY=xxx"));
}

// ========== 工具定义一致性测试 ==========

#[test]
fn test_tools_config_consistency() {
    let tools = routes::load_tools();
    assert_eq!(tools.tools.len(), 3, "应有 3 个工具定义");

    let ids: Vec<&str> = tools.tools.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&"codex"));
    assert!(ids.contains(&"claude"));
    assert!(ids.contains(&"gemini"));

    for tool in &tools.tools {
        assert!(!tool.id.is_empty());
        assert!(!tool.name.is_empty());
        assert!(!tool.config_dir.is_empty());
        assert!(!tool.config_file.is_empty());
        assert!(!tool.format.is_empty());
        assert!(!tool.default_config.is_empty());
        assert!(!tool.install_commands.windows.is_empty());
        assert!(!tool.install_commands.unix.is_empty());

        let format = ConfigFormat::from_str(&tool.format);
        assert!(format.is_some());

        let parsed = ParsedConfig::parse(&tool.default_config, format.unwrap());
        assert!(parsed.is_ok(), "工具 {} 的默认配置应能解析", tool.id);
    }
}

#[test]
fn test_default_config_has_base_url() {
    let tools = routes::load_tools();
    for tool in &tools.tools {
        let format = ConfigFormat::from_str(&tool.format).unwrap();
        let config = ParsedConfig::parse(&tool.default_config, format).unwrap();
        let urls = config.find_base_urls();
        assert!(
            !urls.is_empty(),
            "工具 {} 的默认配置应包含 base_url 字段",
            tool.id
        );
    }
}

#[test]
fn test_preset_routes_have_all_tool_urls() {
    let config = routes::default_user_config();
    let tools = routes::load_tools();

    for preset in &config.preset_routes {
        for tool in &tools.tools {
            let url = preset.urls.get(&tool.id);
            assert!(
                url.is_some(),
                "预设线路「{}」应包含工具 {} 的 URL",
                preset.name,
                tool.id
            );
        }
    }
}
