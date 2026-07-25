# Route Constitution

## Core Principles

### I. 跨平台路径优先 (NON-NEGOTIABLE)

所有文件路径 MUST 使用 `dirs::home_dir()` + `PathBuf::join()` 构建，绝不手动拼接 `~`、`\`、`/` 或硬编码平台路径。三个 CLI 工具在所有平台上统一使用 `~/.<工具名>/` 惯例，无例外。

- `dirs::home_dir()` 返回值：Windows → `C:\Users\<用户名>`，Linux → `/home/<用户名>`，macOS → `/Users/<用户名>`
- 文件权限：Linux/macOS 需在写入前读取 `std::fs::metadata(file).permissions()`，写入后恢复
- 文件名：始终使用官方精确小写文件名，不依赖大小写不敏感性
- 行尾符：统一 `\n`（LF），编码 UTF-8 无 BOM

### II. 安全写入 (NON-NEGOTIABLE)

所有配置文件修改 MUST 遵循备份→写入→失败回滚流程：

1. 写入前：复制原文件为 `<filename>.backup.<timestamp>`
2. 写入失败：删除新文件，重命名备份为原文件名（回滚）
3. 成功后：保留备份文件
4. 权限不足：提示用户手动检查，不静默失败

### III. 精确解析

配置文件 MUST 使用合适的解析方式：

- **JSON**（Claude Code）：`serde_json` crate 解析-序列化，递归搜索 `base_url`
- **TOML**（Codex CLI）：正则表达式替换（保留原始格式），读取时用正则提取
- **ENV**（Gemini CLI）：正则表达式替换（保留原始格式），读取时用正则提取
- `base_url` 搜索不区分大小写，递归遍历所有层级
- 未找到 `base_url` 字段时发出警告，允许用户确认是否继续

### IV. 核心逻辑与框架解耦 (NON-NEGOTIABLE)

核心逻辑（parsers/backup/writer/routes/config_paths）MUST 独立于 Tauri 框架，位于 `crates/core/` library crate 中。Tauri 应用层（`src-tauri/`）仅为薄层 commands，调用 `route_core::` 模块。核心逻辑 MUST 可独立编译和测试（`cargo test -p route-core`），不链接 Tauri 运行时 DLL。

### V. URL-Only 修改

工具 MUST 仅修改配置文件中的 `base_url` 字段值，绝不读取、存储或修改敏感信息（如 API Key、Token）。自定义线路 URL MUST 在写入前进行格式校验（合法 URL scheme + 非空）。

### VI. 用户反馈先行

所有用户操作 MUST 有明确反馈（成功/失败/警告）。关键操作（线路切换）MUST 经确认提示（显示目标地址和说明）后才执行。异常状态（文件不存在、字段未找到、写入失败）MUST 显示具体原因和恢复建议，不静默忽略。

## 技术栈约束

- **桌面框架**：Tauri 2（≥2.11），使用系统 WebView 渲染前端
- **后端语言**：Rust（stable ≥1.77），核心逻辑在 `crates/core/`，Tauri commands 在 `src-tauri/`
- **前端**：原生 HTML + Tailwind CSS + JavaScript（无构建工具，纯静态文件）
- **解析库**：`toml` crate（TOML），`serde_json` crate（JSON），`regex` crate（TOML/ENV 格式保留）
- **数据持久化**：`~/.api-line-switcher/config.json`（JSON 格式，小规模 <100 条线路）
- **配置驱动**：工具定义和预设线路通过 JSON 配置文件管理，不硬编码在源码中
- **不使用**：Electron（体积过大）、Python
- **跨平台测试**：GitHub Actions 三平台（Ubuntu / macOS / Windows），每次 push 自动跑测

## 开发工作流与质量门

- **SPC 流程**：所有功能开发遵循 Specification-Planning-Construction 三阶段流程
- **跨平台测试**：`cargo test -p route-core` 在三平台通过
- **TDD 循环**：对每个任务执行 Red-Green-Refactor
- **路径逻辑测试**：mock `dirs::home_dir()` 验证三平台路径拼接
- **配置解析测试**：各格式（TOML / JSON / ENV）的解析与写入
- **备份回滚测试**：模拟写入失败验证回滚逻辑
- **提交门**：所有测试通过后方可提交，不允许跳过 CI

## Governance

本宪法（Constitution）优先级高于所有其他实践文档。任何原则的修改 MUST 附带变更说明、迁移计划和版本递增（语义化版本）。所有 PR/审查 MUST 验证宪法合规性。

- MAJOR：原则删除或重定义（不兼容变更）
- MINOR：新增原则或实质性扩展
- PATCH：措辞、笔误、非语义性修正

**Version**: 3.0.0 | **Ratified**: 2026-07-25 | **Last Amended**: 2026-07-25

### 变更说明 v2.0.0 → v3.0.0

- **MAJOR**: 技术栈从 "Rust CLI（终端工具）" 变更为 "Tauri 2 跨平台桌面应用"
- **MAJOR**: UI 从 "交互式终端（dialoguer crate）" 变更为 "Tauri WebView（HTML + Tailwind CSS）"
- **MAJOR**: 删除 "不使用 Tauri" 条款
- **MAJOR**: 新增原则 IV — 核心逻辑与框架解耦（`crates/core/` 独立可测）
- **MAJOR**: "单二进制文件" 约束变更为 "Tauri 桌面应用包"
- **MINOR**: TOML/ENV 从 parse-serialize 变更为正则替换（保留原始格式）
- **MINOR**: 新增 SPC 流程约束
