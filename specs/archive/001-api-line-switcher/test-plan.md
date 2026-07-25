# API 线路切换工具 - 测试计划

## 测试目标

全面验证 CLI API Line Switcher 桌面应用的所有功能，确保代码健壮性和用户体验。

## 测试环境

- **操作系统**: Windows 11
- **Node.js**: v24.18.0
- **npm**: 11.16.0
- **工具版本**: Tauri 2.11.3

## 测试分类

### 1. 配置文件读取测试

#### 1.1 TOML 格式 (Codex CLI)
- [ ] 读取简单的 `base_url = "..."` 格式
- [ ] 读取带引号的值 `base_url = '...'`
- [ ] 读取嵌套在表中的 `[section] base_url = "..."`
- [ ] 读取后文件格式不被破坏（字段顺序、注释保留）
- [ ] 读取包含特殊字符的 URL

#### 1.2 JSON 格式 (Claude Code)
- [ ] 读取 `"ANTHROPIC_BASE_URL": "..."`
- [ ] 读取嵌套对象中的 base_url
- [ ] 读取后 JSON 格式正确（缩进、引号）

#### 1.3 ENV 格式 (Gemini CLI)
- [ ] 读取 `GOOGLE_GEMINI_BASE_URL=https://...`
- [ ] 读取带引号的值 `GOOGLE_GEMINI_BASE_URL="https://..."`
- [ ] 读取多行 ENV 文件中的 base_url
- [ ] 使用多行正则模式 `(?im)` 正确匹配

### 2. 配置文件写入测试

#### 2.1 TOML 写入
- [ ] 只修改 base_url 值，不改变其他字段
- [ ] 保留原始字段顺序
- [ ] 保留原始注释
- [ ] 保留原始引号类型（单引号/双引号）
- [ ] 写入前自动创建备份
- [ ] 写入失败自动回滚

#### 2.2 JSON 写入
- [ ] 正确替换 base_url 值
- [ ] 保持 JSON 格式美观（pretty print）
- [ ] 不破坏其他字段

#### 2.3 ENV 写入
- [ ] 只修改包含 BASE_URL 的行
- [ ] 保留其他环境变量
- [ ] 保留行尾空格和注释

### 3. 线路切换功能测试

#### 3.1 预设线路切换
- [ ] 切换到「全球高保」线路
- [ ] 切换到「国内优化」线路
- [ ] 切换后显示正确的线路名称
- [ ] 切换后配置文件正确更新

#### 3.2 自定义线路切换
- [ ] 切换到自定义线路
- [ ] 切换后显示自定义线路名称
- [ ] 切换后配置文件正确更新

#### 3.3 线路名称显示
- [ ] 预设线路显示正确名称（全球高保/国内优化）
- [ ] 自定义线路显示保存时的名称
- [ ] 未知线路显示"自定义"
- [ ] 空 URL 显示"未知"

### 4. 自定义线路管理测试

#### 4.1 添加自定义线路
- [ ] 添加时正确保存 toolId
- [ ] 添加后显示在对应工具下
- [ ] 添加后持久化到 config.json
- [ ] 验证 URL 格式
- [ ] 验证名称非空

#### 4.2 编辑自定义线路
- [ ] 编辑后更新显示
- [ ] 编辑后持久化

#### 4.3 删除自定义线路
- [ ] 删除后从列表移除
- [ ] 删除后持久化

### 5. UI 测试

#### 5.1 按钮显示
- [ ] Codex: 3 个按钮（打开目录、打开 config、打开 auth）
- [ ] Claude: 2 个按钮（打开目录、打开 settings）
- [ ] Gemini: 2 个按钮（打开目录、打开 .env）

#### 5.2 状态显示
- [ ] 已安装工具显示"正常"
- [ ] 未安装工具显示"未检测到"
- [ ] 解析失败显示错误信息
- [ ] 当前线路正确显示

#### 5.3 环境检测
- [ ] 正确检测 Node.js 版本
- [ ] 正确检测 npm 版本
- [ ] Windows 上使用 npm.cmd 或 cmd /c npm

### 6. 文件操作测试

#### 6.1 打开目录
- [ ] 目录存在时正确打开
- [ ] 目录不存在时创建后打开
- [ ] Windows 上使用 explorer 命令

#### 6.2 打开文件
- [ ] 文件存在时正确打开
- [ ] 文件不存在时显示错误

### 7. 边界情况测试

- [ ] 配置文件中无 base_url 字段
- [ ] base_url 值为空字符串
- [ ] 配置文件被其他程序占用
- [ ] 网络代理 URL 包含特殊字符
- [ ] 同时切换多个工具

## 自动化测试脚本

### Rust 单元测试

```rust
// writer.rs 测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_url_toml() {
        let content = r#"base_url = "https://example.com""#;
        assert_eq!(extract_base_url_toml(content), Some("https://example.com".to_string()));
    }

    #[test]
    fn test_extract_base_url_env() {
        let content = r#"GOOGLE_GEMINI_BASE_URL=https://example.com"#;
        assert_eq!(extract_base_url_env(content), Some("https://example.com".to_string()));
    }

    #[test]
    fn test_replace_base_url_toml() {
        let content = r#"base_url = "https://old.com""#;
        let (new_content, count, found) = replace_base_url_toml_regex(content, "https://new.com").unwrap();
        assert!(found);
        assert_eq!(count, 1);
        assert!(new_content.contains("https://new.com"));
    }
}
```

### 集成测试

```javascript
// 前端集成测试
async function testSwitchRoute() {
    const result = await invoke('switch_route', { 
        toolId: 'gemini', 
        targetUrl: 'https://api.example.com/gemini' 
    });
    assert(result.success === true);
    
    // 验证文件内容
    const content = await fs.readFile('~/.gemini/.env', 'utf8');
    assert(content.includes('GOOGLE_GEMINI_BASE_URL=https://api.example.com/gemini'));
}
```

## 测试执行记录

| 测试项 | 状态 | 备注 |
|--------|------|------|
| TOML 读取 | ✅ | 正则表达式提取正常 |
| JSON 读取 | ✅ | serde_json 解析正常 |
| ENV 读取 | ✅ | 多行正则 `(?im)` 修复后正常 |
| TOML 写入 | ✅ | 保留格式 |
| JSON 写入 | ✅ | pretty print |
| ENV 写入 | ✅ | 保留其他行 |
| 预设线路切换 | ✅ | 正常 |
| 自定义线路切换 | ✅ | toolId 参数名修复后正常 |
| 线路名称显示 | ✅ | getCurrentRouteName 修复后正常 |
| 按钮显示 | ✅ | 根据 config_file 动态显示 |
| Node/npm 检测 | ✅ | Windows 上使用 npm.cmd |
| 打开目录 | ✅ | explorer 命令 |
| 打开文件 | ✅ | 正常 |

## 已知问题修复记录

1. **ENV 正则表达式**: 添加 `(?im)` 多行模式
2. **toolId 参数名**: Rust 后端改为驼峰命名 `toolId`
3. **getCurrentRouteName**: 添加 toolId 匹配逻辑
4. **按钮文字**: 根据 config_file 动态显示
5. **npm 检测**: Windows 上使用 `npm.cmd` 或 `cmd /c npm`

## 发布前检查清单

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 手动测试三个工具的完整切换流程
- [ ] 验证备份和回滚功能
- [ ] 验证配置文件格式不被破坏
- [ ] 验证安装包大小 < 20MB
