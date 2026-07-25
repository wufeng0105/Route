# Feature Specification: Tauri 项目重构建

**Feature Branch**: `002-tauri-rebuild`

**Created**: 2026-07-25

**Status**: Draft

**Input**: CLAUDE.md 第一部分（代码库现状分析）+ 第三部分（项目重规划）

## Summary

将 Route 项目从"打补丁式"开发状态重构为按 SPC 流程管理的正式 Tauri 跨平台桌面应用。核心工作包括：拆分核心逻辑与 Tauri 框架（使核心可独立测试）、修复全部 P0/P1 代码缺陷、建立完整的测试体系、更新过期文档。

---

## User Scenarios & Testing

### User Story 1 — 基础设施修复：Workspace 拆分 (Priority: P0)

作为开发者，我希望将核心逻辑（parsers/backup/writer/routes/config_paths）从 Tauri crate 中拆分到独立的 `crates/core/` library crate 中，使得 `cargo test` 无需 Tauri 运行时 DLL 即可正常运行，同时 Tauri 应用层变为仅包含 commands 薄层。

**Why this priority**: 当前 38 个单元测试全部无法运行（Tauri DLL 依赖导致 `STATUS_ENTRYPOINT_NOT_FOUND`），这是所有后续开发的前提条件。不解决此问题，无法验证任何代码变更的正确性。

**Independent Test**: 拆分完成后执行 `cargo test -p route-core`，所有原有单元测试通过；执行 `cargo build -p app`，Tauri 应用正常编译。

**Acceptance Scenarios**:

1. **Given** 项目当前结构为 `src-tauri/src/` 包含所有模块，**When** 创建 Cargo workspace 并拆分出 `crates/core/`，**Then** `cargo test -p route-core` 编译并运行成功，38 个测试全部通过
2. **Given** core crate 不依赖 tauri，**When** 执行 `cargo tree -p route-core`，**Then** 依赖树中不包含 `tauri` 或 `tauri-build`
3. **Given** Tauri 应用层 `src-tauri/src/lib.rs`，**When** 编译 `cargo build -p app`，**Then** 编译成功，Tauri commands 正常调用 `core::` 模块
4. **Given** CI 工作流，**When** push 到 main 分支，**Then** GitHub Actions 在三平台（Ubuntu/macOS/Windows）上执行 `cargo test -p route-core` 和 `cargo build -p app` 均通过

---

### User Story 2 — 代码缺陷修复 (Priority: P1)

作为开发者，我希望修复 CLAUDE.md 中列出的全部 P1 级代码缺陷（共 6 项），使得项目的核心逻辑正确无误。

**Why this priority**: P1 缺陷影响正确性——URL 校验 Bug 会导致无效 URL 被接受，格式处理不一致会破坏用户配置文件，前端 XSS 风险可能被利用。必须在功能开发前修复。

**Independent Test**: 修复完成后执行 `cargo test -p route-core`，新增的回归测试全部通过；前端 `escape()` 函数手动验证单引号转义。

**Acceptance Scenarios**:

1. **Given** `is_valid_url("http://")` 当前返回 `true`（Bug），**When** 修复运算符优先级为 `(a || b) && c`，**Then** `is_valid_url("http://")` 返回 `false`，`is_valid_url("https://api.example.com")` 返回 `true`
2. **Given** `write_default_config` 对 TOML 用 parse-serialize（会重排键序），**When** 统一为正则替换方式（与 `safe_write` 一致），**Then** 已有 TOML 配置的键顺序在写入后保持不变
3. **Given** "管理线路"弹窗的"添加新线路"按钮调用 `openAddRoute()` 无参数，**When** 修复为弹出工具选择或从管理上下文传递 toolId，**Then** `pendingAddToolId` 不再为 `undefined`
4. **Given** `install_tool` 标记为 `async` 但无 `await`，**When** 改用 `spawn_blocking` 或移除 `async`，**Then** 安装操作不阻塞 UI 线程
5. **Given** 前端 `escape()` 不转义单引号，**When** 添加 `'` → `&#39;` 转义，**Then** URL 含单引号时无法注入 `onclick` 属性
6. **Given** `handleInstall` 使用浏览器 `prompt()`，**When** 替换为自定义 Tauri modal 弹窗，**Then** 线路选择在 macOS/Linux WebView 中正常工作

---

### User Story 3 — 测试体系建设 (Priority: P0)

作为开发者，我希望建立完整的集成测试体系，覆盖 `safe_write` 全格式端到端流程、备份回滚机制、线路管理 CRUD 操作，使得核心写入流程有自动化验证保障。

**Why this priority**: 当前 0 个集成测试，核心写入流程（用户切换线路 → 备份 → 替换 → 写入 → 失败回滚）完全无自动化验证。这是 P0 级别，因为任何代码变更都可能破坏核心流程而无人知晓。

**Independent Test**: 执行 `cargo test -p route-core --test integration_tests`，所有集成测试通过。

**Acceptance Scenarios**:

1. **Given** 一个临时 TOML 文件包含 `base_url = "https://old.example.com"`，**When** 调用 `safe_write` 切换到新 URL，**Then** 文件内容更新为新 URL，备份文件存在且包含旧 URL，其他字段（model、api_key）保留
2. **Given** 一个临时 JSON 文件包含 `ANTHROPIC_BASE_URL`，**When** 调用 `safe_write` 切换 URL，**Then** JSON 文件更新，其他字段保留
3. **Given** 一个临时 ENV 文件包含 `GOOGLE_GEMINI_BASE_URL=`，**When** 调用 `safe_write` 切换 URL，**Then** ENV 文件更新，注释和其他键值对保留
4. **Given** 备份文件已创建且原文件被破坏，**When** 调用 `rollback`，**Then** 原文件恢复为备份内容，备份文件被重命名为原文件名
5. **Given** 空的 `customRoutes` 列表，**When** 调用 `add_custom_route`，**Then** 列表长度变为 1，调用 `delete_custom_route` 后恢复为 0
6. **Given** 同一工具下已有名为"测试线路"的自定义线路，**When** 再次添加同名线路，**Then** 操作被拒绝并返回错误

---

### User Story 4 — 文档体系建立 (Priority: P1)

作为项目维护者，我希望建立完整的文档体系，包括 README、规格文档、实施计划和任务清单，使得项目可维护、可追溯、对新贡献者友好。

**Why this priority**: 当前无 README，Constitution 过期（仍引用 CLI 架构），specs/ 目录含过期文档。文档不完善会影响项目长期可维护性。

**Independent Test**: 新开发者按照 README 的构建指南，能在 15 分钟内从零构建并运行项目。

**Acceptance Scenarios**:

1. **Given** 项目根目录无 README.md，**When** 编写 README 含项目说明、前置条件、构建步骤、使用方法，**Then** 新用户按 README 可独立完成构建
2. **Given** Constitution 仍引用 CLI 架构（dialoguer、终端 UI），**When** 更新为 Tauri 桌面应用架构，**Then** Constitution 版本升至 3.0.0，技术栈描述与实际一致
3. **Given** `specs/001-api-line-switcher/` 含过期 CLI 规格文档，**When** 归档或删除，**Then** specs/ 目录只包含当前有效的规格文档
4. **Given** 项目根目录无 `.gitignore`，**When** 创建含 `target/`、`*.log` 等规则，**Then** 构建产物不被提交到 Git

---

### User Story 5 — 前端健壮性 (Priority: P2)

作为用户，我希望应用在离线环境下 UI 正常显示，编辑线路操作独立且稳定，Tauri API 不可用时有明确提示而非静默降级。

**Why this priority**: P2 级别——功能可用但有体验缺陷。CDN 依赖导致离线时 UI 完全失效，编辑弹窗复用逻辑脆弱可能导致误操作。

**Independent Test**: 断开网络后启动应用，UI 正常显示；在"管理线路"弹窗中编辑线路后关闭，再次打开"添加线路"弹窗，行为正常。

**Acceptance Scenarios**:

1. **Given** 前端从 CDN 加载 Tailwind CSS、Google Fonts、Material Symbols，**When** 本地化这些资源到 `frontend/assets/`，**Then** 断网后 UI 样式正常
2. **Given** 编辑线路复用"添加"弹窗并动态替换 `onclick`，**When** 改为独立编辑弹窗，**Then** 关闭编辑弹窗后"添加"弹窗行为不受影响
3. **Given** "管理线路"弹窗描述写"所有工具共享同一 URL"，**When** 修正为"每个工具独立配置自定义线路"，**Then** 描述与实际行为一致
4. **Given** Tauri API 不可用时静默降级到 Mock 数据，**When** 改为显式错误提示，**Then** 用户看到"无法连接到后端服务"而非假数据

---

### Edge Cases

- 配置文件为空文件时 `safe_write` 的行为？→ 应返回错误"未找到 base_url 字段"
- 配置文件包含多个 `base_url` 字段（嵌套）时？→ 全部替换，`replaced_count` 反映实际数量
- `load_user_config` 解析 config.json 失败时？→ 保留自定义线路（best-effort），使用默认预设线路
- 跨平台路径中 Home 目录无法确定时？→ 所有路径函数返回 `None`/`Err`，不 panic
- 备份文件名时间戳冲突（同一秒内多次写入）时？→ 后一次覆盖前一次备份（可接受）

---

## Requirements

### Functional Requirements

- **FR-001**: 系统 MUST 拆分为 Cargo workspace，核心逻辑在 `crates/core/`，Tauri 应用在 `src-tauri/`
- **FR-002**: `crates/core/` MUST 不依赖 `tauri` 或 `tauri-build` crate
- **FR-003**: `src-tauri/src/lib.rs` MUST 通过 `use route_core::...` 引用核心模块
- **FR-004**: `cargo test -p route-core` MUST 编译并运行成功，原有 38 个测试全部通过
- **FR-005**: `cargo build -p app` MUST 编译成功
- **FR-006**: CI 工作流 MUST 在三平台执行 `cargo test -p route-core` 和 `cargo build -p app`
- **FR-007**: `is_valid_url` MUST 使用 `(starts_with("http://") || starts_with("https://")) && len > 8` 逻辑
- **FR-008**: `write_default_config` MUST 对 TOML/ENV 使用正则替换（与 `safe_write` 一致）
- **FR-009**: "管理线路"弹窗的"添加新线路"按钮 MUST 传递有效 toolId 或弹出工具选择
- **FR-010**: `install_tool` MUST 不阻塞 UI 线程
- **FR-011**: 前端 `escape()` MUST 转义单引号 `'` → `&#39;`
- **FR-012**: `handleInstall` MUST 使用自定义 modal 替代 `prompt()`
- **FR-013**: 集成测试 MUST 覆盖 `safe_write` 全格式（TOML/JSON/ENV）端到端
- **FR-014**: 集成测试 MUST 覆盖备份创建与回滚流程
- **FR-015**: 集成测试 MUST 覆盖线路管理 CRUD（增删改 + 校验）
- **FR-016**: 项目根 MUST 存在 `.gitignore` 忽略 `target/` 等
- **FR-017**: 项目根 MUST 存在 `README.md` 含构建指南
- **FR-018**: Constitution MUST 升级至 3.0.0，反映 Tauri 架构
- **FR-019**: `tauri.conf.json` 的 identifier MUST 改为 `com.route.app`
- **FR-020**: `build.ps1` MUST 不硬编码用户路径

### Key Entities

- **ToolDef**: 工具定义（id, name, config_dir, config_file, format, install_commands, default_config）
- **UserConfig**: 用户配置（preset_routes: Vec<PresetRoute>, custom_routes: Vec<CustomRoute>）
- **CustomRoute**: 自定义线路（tool_id, name, url）— 每个线路绑定一个工具
- **SwitchResult**: 切换结果（success, tool_name, target_url, backup_path, replaced_count, base_url_found, error）

---

## Success Criteria

### Measurable Outcomes

- **SC-001**: `cargo test -p route-core` 在三平台通过，38+ 个单元测试 + 集成测试全部绿色
- **SC-002**: `cargo build -p app` 在三平台编译成功，无 warning
- **SC-003**: `cargo clippy -p route-core -- -D warnings` 无警告
- **SC-004**: 新开发者按 README 构建指南，15 分钟内从零完成构建并运行
- **SC-005**: 断网后启动应用，UI 样式正常显示（CDN 资源已本地化）

---

## Assumptions

- 用户已安装 Rust stable (≥1.77) 和 Tauri 2 CLI
- Windows 构建需要 GNU 工具链 + MinGW（提供 as.exe/dlltool.exe）
- 前端保持纯静态 HTML + JS，不引入构建工具（如 Vite/webpack）
- 预设线路 URL（`aicodemirror.ai`、`claudecode.net.cn`）保持不变
- 集成测试使用临时文件，不修改用户实际配置
- `specs/001-api-line-switcher/` 旧规格文档将被归档（不删除，移至 `specs/archive/`）
