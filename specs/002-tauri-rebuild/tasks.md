# Tasks: Tauri 项目重构建

**Input**: Design documents from `/specs/002-tauri-rebuild/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 项目初始化和基础结构

- [ ] T001 [P] [US4] 创建 `.gitignore` 在项目根，忽略 `target/`、`*.log`、`.env`、`node_modules/`
- [ ] T002 [P] [US4] 创建 `specs/archive/` 目录，移动 `specs/001-api-line-switcher/` 到其中
- [ ] T003 [P] [US4] 创建 `README.md` 在项目根，含项目说明、前置条件、构建步骤、使用方法

---

## Phase 2: Foundational — Workspace 拆分 (US1)

**Purpose**: Cargo workspace 拆分，核心逻辑独立为 library crate

**⚠️ CRITICAL**: 所有后续任务依赖此阶段完成

### Workspace 配置

- [ ] T004 [US1] 创建 Workspace `Cargo.toml` 在项目根，内容:
  ```toml
  [workspace]
  members = ["crates/core", "src-tauri"]
  resolver = "2"
  ```
- [ ] T005 [US1] 创建 `crates/core/Cargo.toml`，包名 `route-core`，依赖: serde, serde_json, toml, dirs, regex（无 tauri）

### 模块迁移

- [ ] T006 [US1] 创建 `crates/core/src/lib.rs`，内容:
  ```rust
  pub mod backup;
  pub mod config_paths;
  pub mod parsers;
  pub mod routes;
  pub mod writer;
  ```
- [ ] T007 [P] [US1] 移动 `src-tauri/src/config_paths.rs` → `crates/core/src/config_paths.rs`
- [ ] T008 [P] [US1] 移动 `src-tauri/src/backup.rs` → `crates/core/src/backup.rs`
- [ ] T009 [P] [US1] 移动 `src-tauri/src/routes.rs` → `crates/core/src/routes.rs`
- [ ] T010 [P] [US1] 移动 `src-tauri/src/writer.rs` → `crates/core/src/writer.rs`
- [ ] T011 [P] [US1] 移动 `src-tauri/src/parsers/` 目录 → `crates/core/src/parsers/`
- [ ] T012 [P] [US1] 移动 `src-tauri/src/tools.json` → `crates/core/src/tools.json`

### 模块路径修正

- [ ] T013 [US1] 修正 `crates/core/src/` 内所有 `use crate::` 引用（保持不变，因为模块在同一 crate 内）
- [ ] T014 [US1] 修改 `src-tauri/src/lib.rs`：删除 `mod backup; mod config_paths; ...` 声明，改为 `use route_core::...`

### Tauri 配置修正

- [ ] T015 [US1] 修改 `src-tauri/Cargo.toml`：添加 `route-core = { path = "../crates/core" }` 依赖，移除 serde/serde_json/toml/dirs/regex 直接依赖（通过 core 间接使用）
- [ ] T016 [P] [US1] 修改 `tauri.conf.json`：`identifier` 改为 `com.route.app`，`title` 改为 `Route 线路切换工具`
- [ ] T017 [P] [US1] 修改 `build.ps1`：将硬编码 `$MingwBin` 改为动态检测 `scoop prefix mingw` 或环境变量

### 验证

- [ ] T018 [US1] 执行 `cargo test -p route-core`，验证 38 个原有单元测试全部通过
- [ ] T019 [US1] 执行 `cargo build -p app`，验证 Tauri 应用编译成功
- [ ] T020 [US1] 执行 `cargo tree -p route-core`，验证依赖树不含 `tauri`

**Checkpoint**: core crate 独立可测，Tauri 应用层编译成功

---

## Phase 3: CI 工作流修复 (US1)

**Purpose**: CI 在三平台正确执行 workspace 测试和构建

- [ ] T021 [US1] 修改 `.github/workflows/test.yml`：
  - Build 步骤改为 `cargo build --verbose`
  - Test 步骤改为 `cargo test -p route-core --verbose`（只测 core，避免 Tauri DLL 问题）
  - Clippy 步骤改为 `cargo clippy -p route-core -- -D warnings`
  - Cache 路径改为 `~/.cargo/registry` + `target`（workspace 级）

**Checkpoint**: CI 可在三平台通过

---

## Phase 4: 代码缺陷修复 (US2)

**Purpose**: 修复 P1 级代码缺陷

### Bug 修复

- [ ] T022 [US2] 修复 `crates/core/src/routes.rs` `is_valid_url`：
  - 改为 `(url.starts_with("http://") || url.starts_with("https://")) && url.len() > 8`
  - 添加测试: `assert!(!is_valid_url("http://"))` 和 `assert!(is_valid_url("https://a.b"))`

- [ ] T023 [US2] 统一 `crates/core/src/writer.rs` `write_default_config`：
  - 对 TOML 格式使用 `replace_base_url_toml_regex`（与 `safe_write` 一致）
  - 对 ENV 格式使用 `replace_base_url_env_regex`（与 `safe_write` 一致）
  - 仅在正则替换失败时回退到 parse-serialize

- [ ] T024 [US2] 修复 `frontend/index.html:273` `openAddRoute()` 调用：
  - 改为先弹出工具选择列表，或从管理弹窗上下文传递选中的工具 ID

- [ ] T025 [US2] 修复 `src-tauri/src/lib.rs` `install_tool`：
  - 移除 `async` 标记，或改用 `tauri::async_runtime::spawn_blocking` 包装同步调用

- [ ] T026 [P] [US2] 修复 `frontend/app.js` `escape()` 函数：
  - 添加 `.replace(/'/g, '&#39;')` 转义单引号

- [ ] T027 [US2] 替换 `frontend/app.js` `handleInstall` 中的 `prompt()`：
  - 在 `index.html` 中添加线路选择 modal
  - `app.js` 中打开 modal 代替 `prompt()`

### 验证

- [ ] T028 [US2] 执行 `cargo test -p route-core`，验证 `is_valid_url` 新测试通过
- [ ] T029 [US2] 手动验证前端 `escape()` 转义单引号

**Checkpoint**: P1 缺陷全部修复

---

## Phase 5: 测试体系建设 (US3)

**Purpose**: 建立集成测试覆盖核心流程

### 集成测试编写

- [ ] T030 [US3] 创建 `tests/integration_tests.rs`，编写 `safe_write` TOML 端到端测试：
  - 创建临时 TOML 文件 → 调用 `safe_write` → 验证文件内容更新、备份文件存在、其他字段保留

- [ ] T031 [P] [US3] 编写 `safe_write` JSON 端到端测试（同上，JSON 格式）

- [ ] T032 [P] [US3] 编写 `safe_write` ENV 端到端测试（同上，ENV 格式）

- [ ] T033 [US3] 编写备份+回滚完整流程测试：
  - 创建文件 → 创建备份 → 破坏原文件 → 调用 `rollback` → 验证恢复

- [ ] T034 [US3] 编写线路管理 CRUD 测试：
  - `add_custom_route` → `edit_custom_route` → `delete_custom_route`
  - 同名拒绝、无效 URL 拒绝、越界索引拒绝

- [ ] T035 [P] [US3] 编写工具定义一致性测试：
  - `load_tools` → 验证 3 个工具都有必要字段
  - `default_user_config` → 验证预设线路包含所有工具 URL

### 验证

- [ ] T036 [US3] 执行 `cargo test -p route-core --test integration_tests`，全部通过
- [ ] T037 [US3] CI 中添加 `cargo test -p route-core --test integration_tests` 步骤

**Checkpoint**: 核心流程有自动化验证保障

---

## Phase 6: 文档体系建立 (US4)

**Purpose**: 建立完整文档体系

- [ ] T038 [US4] 更新 `.specify/memory/constitution.md`：
  - 版本升至 3.0.0
  - 技术栈改为 Tauri 2 + Rust + 静态 HTML
  - 删除"不使用 Tauri"条款
  - UI 描述改为 WebView GUI

- [ ] T039 [P] [US4] 清理 `stitch_*.html`：移动到 `design/` 目录或删除

- [ ] T040 [US4] 验证 README.md 构建指南可操作性

**Checkpoint**: 文档与代码一致

---

## Phase 7: 前端健壮性 (US5)

**Purpose**: 离线可用、交互健壮

- [ ] T041 [P] [US5] 本地化 Tailwind CSS：下载 `tailwind.min.css` 到 `frontend/assets/`
- [ ] T042 [P] [US5] 本地化 Google Fonts（Inter + JetBrains Mono）到 `frontend/assets/fonts/`
- [ ] T043 [P] [US5] 本地化 Material Symbols Outlined 到 `frontend/assets/icons/`
- [ ] T044 [US5] 修改 `frontend/index.html`：CDN 引用改为本地路径
- [ ] T045 [US5] 修复编辑线路弹窗：创建独立编辑 modal，不复用添加弹窗
- [ ] T046 [US5] 修复 `frontend/index.html:268` "管理线路"弹窗描述：改为"每个工具独立配置自定义线路"
- [ ] T047 [US5] 修复 `frontend/app.js` Mock 数据：Tauri API 不可用时显示错误提示

**Checkpoint**: 离线 UI 正常，交互稳定

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ────→ Phase 2 (Workspace 拆分) ────→ Phase 3 (CI 修复)
                                          │
                                          ├──→ Phase 4 (Bug 修复)
                                          │
                                          ├──→ Phase 5 (测试体系)
                                          │
                                          └──→ Phase 6 (文档体系)

Phase 4/5/6 完成后 ────→ Phase 7 (前端健壮性)
```

### Critical Path

T004 → T005 → T006 → T007-T012 (并行) → T013 → T014 → T015 → T018 → T019

### Parallel Opportunities

- T007-T012: 6 个文件移动可并行
- T016, T017: Tauri 配置修改可并行
- T022-T027: Bug 修复可并行（不同文件）
- T030-T035: 集成测试编写可并行
- T041-T043: CDN 本地化可并行

---

## Implementation Strategy

### MVP First (Phase 1-3)

1. 完成 Setup（Phase 1）
2. 完成 Workspace 拆分（Phase 2）
3. 完成 CI 修复（Phase 3）
4. **STOP and VALIDATE**: `cargo test -p route-core` 三平台通过

### Incremental Delivery

1. Phase 1-3 → 基础设施就绪（可测试、可 CI）
2. Phase 4 → 代码缺陷修复
3. Phase 5 → 测试体系
4. Phase 6 → 文档完善
5. Phase 7 → 前端健壮性

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- 每个任务完成后 commit
- Phase 2 是关键路径，必须先完成
- 集成测试（Phase 5）依赖 Phase 2 的 core crate 拆分
- 前端任务（Phase 4/7）可与后端任务并行
