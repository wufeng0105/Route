//! Claude Code 端到端测试
//!
//! 在真实安装了 Claude Code 的环境中验证：
//! 1. 配置文件路径是否正确
//! 2. safe_write 能否正确替换 base_url
//! 3. 其他字段（如 API Key）是否保留
//!
//! 用法：cargo run -p route-core --example claude_code_e2e

use route_core::config_paths;
use route_core::parsers::ConfigFormat;
use route_core::writer;
use std::fs;

fn main() {
    println!("=== Claude Code 端到端测试 ===\n");

    // 1. 查找配置文件路径
    println!("【步骤 1】查找 Claude Code 配置文件路径...");
    let config_path = config_paths::get_tool_config_path(".claude", "settings.json");

    match &config_path {
        Some(path) => {
            println!("  ✓ 配置路径: {}", path.display());

            if !path.exists() {
                println!("  ✗ 配置文件不存在（Claude Code 可能未安装或未初始化）");
                println!("\n  尝试查找其他可能的配置文件...");
                // 列出 ~/.claude/ 目录内容
                if let Some(claude_dir) = config_paths::get_tool_config_dir(".claude") {
                    if claude_dir.exists() {
                        println!("  ~/.claude/ 目录内容:");
                        if let Ok(entries) = fs::read_dir(&claude_dir) {
                            for entry in entries.flatten() {
                                let name = entry.file_name();
                                let size = entry.metadata().map(|m| {
                                    if m.is_file() {
                                        format!("{} bytes", m.len())
                                    } else {
                                        "<dir>".to_string()
                                    }
                                }).unwrap_or_else(|_| "<unknown>".to_string());
                                println!("    {} ({})", name.to_string_lossy(), size);
                            }
                        }
                    } else {
                        println!("  ~/.claude/ 目录不存在");
                    }
                }
                return;
            }
        }
        None => {
            println!("  ✗ 无法确定 Home 目录");
            return;
        }
    }

    let config_path = config_path.unwrap();

    // 2. 显示原始配置文件内容
    println!("\n【步骤 2】读取原始配置文件内容...");
    let original_content = match fs::read_to_string(&config_path) {
        Ok(content) => {
            println!("  ✓ 读取成功，内容如下：\n");
            // 格式化打印 JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                    println!("  {}", pretty);
                } else {
                    println!("  {}", content);
                }
            } else {
                println!("  {}", content);
            }
            content
        }
        Err(e) => {
            println!("  ✗ 读取失败: {}", e);
            return;
        }
    };

    // 3. 查找当前 base_url
    println!("\n【步骤 3】查找当前 base_url...");
    let format = ConfigFormat::Json;
    match writer::read_base_url(&config_path, format) {
        Ok(Some(url)) => println!("  ✓ 当前 base_url: {}", url),
        Ok(None) => println!("  ⚠ 未找到 base_url 字段（可能尚未配置）"),
        Err(e) => println!("  ✗ 读取失败: {}", e),
    }

    // 4. 使用 safe_write 替换 URL
    let test_url = "https://test-proxy.example.com/v1";
    println!("\n【步骤 4】使用 safe_write 替换 base_url 为: {}", test_url);

    let result = writer::safe_write(&config_path, format, test_url, "Claude Code");

    println!("\n  切换结果:");
    println!("    success: {}", result.success);
    println!("    tool_name: {}", result.tool_name);
    println!("    target_url: {}", result.target_url);
    println!("    base_url_found: {}", result.base_url_found);
    println!("    replaced_count: {}", result.replaced_count);
    println!(
        "    backup_path: {}",
        result
            .backup_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "None".to_string())
    );
    if let Some(ref err) = result.error {
        println!("    error: {}", err);
    }

    if !result.success {
        println!("\n  ✗ 替换失败，测试终止");
        return;
    }

    // 5. 显示替换后的配置文件内容
    println!("\n【步骤 5】读取替换后的配置文件内容...");
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            println!("  ✓ 替换后内容：\n");
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                    println!("  {}", pretty);
                } else {
                    println!("  {}", content);
                }
            } else {
                println!("  {}", content);
            }

            // 验证关键点
            println!("\n【步骤 6】验证结果：");
            let has_new_url = content.contains(test_url);
            let has_old_url = original_content.contains("base_url")
                && original_content != content;
            let preserves_other_fields = original_content
                .lines()
                .filter(|line| !line.contains("base_url") && !line.contains("BASE_URL"))
                .all(|line| content.contains(line.trim_end_matches([' ', ','])));

            println!("  {} 包含新 URL: {}", if has_new_url { "✓" } else { "✗" }, if has_new_url { "是" } else { "否" });
            println!("  {} 旧 URL 已替换: {}", if has_old_url { "✓" } else { "✗" }, if has_old_url { "是" } else { "否" });
            println!("  {} 其他字段保留: {}", if preserves_other_fields { "✓" } else { "⚠" }, if preserves_other_fields { "是" } else { "需检查" });
        }
        Err(e) => println!("  ✗ 读取失败: {}", e),
    }

    // 6. 恢复原始配置
    if let Some(ref backup_path) = result.backup_path {
        println!("\n【步骤 7】从备份恢复原始配置...");
        match fs::read_to_string(backup_path) {
            Ok(backup_content) => {
                fs::write(&config_path, backup_content).ok();
                println!("  ✓ 已从备份恢复: {}", backup_path.display());
            }
            Err(e) => println!("  ⚠ 恢复失败: {}（备份仍在: {}）", e, backup_path.display()),
        }
    }

    println!("\n=== 测试完成 ===\n");
}
