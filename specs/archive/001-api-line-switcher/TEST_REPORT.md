# API 线路切换工具 - 测试报告

## 测试概述

**测试日期**: 2026-07-25  
**测试版本**: v0.1.0  
**测试环境**: Windows 11 + PowerShell 7  
**Node.js**: v24.18.0  
**npm**: 11.16.0  

---

## 1. 启动行为测试

### 1.1 黑框问题分析

**现象**: 启动应用时先出现一个黑色控制台窗口，然后才显示主窗口。

**原因**: 
- `src-tauri/src/main.rs` 中使用了 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
- 这意味着只有在 **release 模式** 下才会隐藏控制台窗口
- 在 **debug 模式** 下，Windows 会显示控制台窗口用于调试输出

**解决方案**:
```rust
// 当前代码（仅在 release 模式隐藏控制台）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 如果要在 debug 模式也隐藏控制台，改为：
#![windows_subsystem = "windows"]
```

**建议**: 保持现状，因为：
1. Debug 模式需要控制台查看日志
2. Release 构建时不会显示黑框
3. 这是 Tauri 的标准做法

---

## 2. 功能测试

### 2.1 配置文件读取

| 测试项 | 状态 | 备注 |
|--------|------|------|
| TOML 格式读取 (Codex) | ✅ | 正则表达式提取正常 |
| JSON 格式读取 (Claude) | ✅ | serde_json 解析正常 |
| ENV 格式读取 (Gemini) | ✅ | 多行正则 `(?im)` 修复后正常 |
| 文件不存在处理 | ✅ | 返回 config_exists = false |
| 解析错误处理 | ✅ | 返回 error 字段 |

### 2.2 配置文件写入

| 测试项 | 状态 | 备注 |
|--------|------|------|
| TOML 格式保留 | ✅ | 使用正则替换，保留注释和格式 |
| JSON 格式美化 | ✅ | pretty print 输出 |
| ENV 格式保留 | ✅ | 保留其他环境变量 |
| 备份创建 | ✅ | 自动创建 `.backup.<timestamp>` |
| 写入失败回滚 | ✅ | 自动回滚到备份 |

### 2.3 线路切换

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 全球高保线路切换 | ✅ | 所有工具正常 |
| 国内优化线路切换 | ✅ | 所有工具正常 |
| 自定义线路切换 | ✅ | 需先添加自定义线路 |
| 切换确认弹窗 | ✅ | 显示工具名、线路名、URL |
| 切换后状态刷新 | ✅ | 自动刷新显示 |

### 2.4 自定义线路管理

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 添加自定义线路 | ✅ | 参数名 `toolId` 修复后正常 |
| 编辑自定义线路 | ✅ | 正常 |
| 删除自定义线路 | ✅ | 正常 |
| 按工具隔离 | ✅ | 每个工具有独立的自定义线路 |
| 持久化存储 | ✅ | 保存到 `~/.api-line-switcher/config.json` |
| URL 格式验证 | ✅ | 必须以 http:// 或 https:// 开头 |
| 名称非空验证 | ✅ | 正常 |

### 2.5 UI 显示

| 测试项 | 状态 | 备注 |
|--------|------|------|
| Codex 按钮 (3个) | ✅ | 打开目录、打开 config、打开 auth |
| Claude 按钮 (2个) | ✅ | 打开目录、打开 settings |
| Gemini 按钮 (2个) | ✅ | 打开目录、打开 .env |
| 状态显示 | ✅ | 正常/未检测到/解析失败 |
| 当前线路名称 | ✅ | 预设/自定义/未知 |
| 环境状态 | ✅ | Node.js/npm 版本显示 |

### 2.6 文件操作

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 打开目录 | ✅ | Windows 使用 explorer |
| 打开配置文件 | ✅ | 使用系统默认程序 |
| 目录不存在创建 | ✅ | 自动创建 |
| 文件不存在提示 | ✅ | 显示错误信息 |

### 2.7 环境检测

| 测试项 | 状态 | 备注 |
|--------|------|------|
| Node.js 检测 | ✅ | 正常 |
| npm 检测 (Windows) | ✅ | 使用 `npm.cmd` 或 `cmd /c npm` |
| 版本显示 | ✅ | 正常 |

---

## 3. 代码健壮性检查

### 3.1 错误处理

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 配置文件不存在 | ✅ | 返回友好错误 |
| 配置文件解析失败 | ✅ | 返回错误详情 |
| 写入权限不足 | ✅ | 返回错误信息 |
| 备份失败 | ✅ | 阻止写入操作 |
| 回滚失败 | ✅ | 报告双重错误 |
| 网络错误（安装） | ✅ | 显示安装命令 |

### 3.2 边界情况

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 空 URL | ✅ | 显示为 "(空)" |
| URL 不匹配任何线路 | ✅ | 显示为 "自定义" |
| 多个 base_url 字段 | ✅ | 全部替换 |
| 特殊字符 URL | ✅ | 正常处理 |
| 超长 URL | ✅ | 正常显示 |

### 3.3 数据验证

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 自定义线路名称重复 | ✅ | 同工具下禁止重复 |
| URL 格式验证 | ✅ | 必须 http/https 开头 |
| 索引越界检查 | ✅ | 编辑/删除时检查 |
| 工具 ID 验证 | ✅ | 切换时验证存在性 |

---

## 4. 已知问题修复记录

### 修复 1: ENV 正则表达式
**问题**: Gemini 的 `.env` 文件无法正确读取 `GOOGLE_GEMINI_BASE_URL`
**原因**: 正则表达式没有使用多行模式
**修复**: 添加 `(?im)` 多行模式标志
```rust
// 修复前
let re = regex::Regex::new(r#"^\s*\w*BASE_URL\w*\s*=\s*["']?([^"'\n]+)["']?\s*$"#).unwrap();

// 修复后
let re = regex::Regex::new(r#"(?im)^\s*\w*BASE_URL\w*\s*=\s*["']?([^"'\n]+)["']?\s*$"#).unwrap();
```

### 修复 2: toolId 参数名
**问题**: 添加自定义线路时，Gemini 和 Claude 的线路显示为"未知"
**原因**: Rust 后端使用 `tool_id` 但前端传递 `toolId`
**修复**: 统一使用驼峰命名 `toolId`
```rust
// 修复前
fn add_custom_route(tool_id: String, ...)

// 修复后
fn add_custom_route(toolId: String, ...)
```

### 修复 3: 线路名称匹配
**问题**: 自定义线路匹配时没有检查 toolId
**原因**: `getCurrentRouteName` 只比较 URL
**修复**: 添加 toolId 匹配
```javascript
// 修复前
if (route.url === tool.current_url) { ... }

// 修复后
if (route.toolId === tool.id && route.url === tool.current_url) { ... }
```

### 修复 4: 按钮显示
**问题**: 所有工具都显示 3 个按钮
**原因**: 按钮数量硬编码
**修复**: 根据工具 ID 动态显示
```javascript
const hasAuthFile = tool.id === 'codex';
const configButtonText = tool.config_file === '.env' ? '.env' : 
                         tool.config_file === 'settings.json' ? 'settings' : 
                         tool.config_file;
```

### 修复 5: npm 检测 (Windows)
**问题**: Windows 上 npm 检测失败
**原因**: Tauri 运行时环境变量与用户 shell 不一致
**修复**: 尝试多种方式检测
```rust
let npm_version = if cfg!(target_os = "windows") {
    Command::new("npm.cmd")
        .arg("--version")
        .output()
        .ok()
        .or_else(|| {
            Command::new("cmd")
                .args(&["/c", "npm", "--version"])
                .output()
                .ok()
        })
}
```

### 修复 6: 打开目录误报错误
**问题**: Windows 上打开目录后报错但实际成功
**原因**: `explorer` 命令返回非零退出码
**修复**: 忽略退出码
```rust
match result {
    Ok(_) => Ok(()), // 忽略退出码
    Err(e) => Err(format!("无法打开: {}", e)),
}
```

---

## 5. 测试执行命令

### Rust 单元测试
```bash
cd src-tauri
cargo test
```

### 集成测试
```bash
# 启动应用并手动测试
./src-tauri/target/debug/app.exe
```

### 构建 Release 版本
```bash
cd src-tauri
cargo build --release
```

---

## 6. 发布前检查清单

- [x] 所有单元测试通过
- [x] 所有集成测试通过
- [x] 手动测试三个工具的完整切换流程
- [x] 验证备份和回滚功能
- [x] 验证配置文件格式不被破坏
- [x] 验证 Release 构建无黑框
- [ ] 验证安装包大小 < 20MB

---

## 7. 结论

**整体状态**: ✅ 可用

所有核心功能已修复并验证通过。代码健壮性良好，错误处理完善。建议在 Release 模式下构建后使用，以避免启动时的黑框。
