# Tasks: CLI API Line Switcher

**Input**: Design documents from `/specs/001-api-line-switcher/`

**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create `Cargo.toml` with dependencies: serde, serde_json, toml, dirs, dialoguer, colored
- [x] T002 Create `src/tools.json` with three tool definitions (codex/claude/gemini)
- [x] T003 [P] Create `.github/workflows/test.yml` for cross-platform CI

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Implement `src/config_paths.rs` — `get_home_dir()`, `get_tool_config_path(tool_id)`, `get_user_config_path()`, `ensure_user_config_dir()`
- [x] T005 [P] Implement `src/parsers/mod.rs` — `ConfigParser` trait, `ParsedConfig` struct, `detect_format()`, `get_parser()`
- [x] T006 [P] Implement `src/parsers/json.rs` — `JsonParser`: parse, find_base_url (recursive), replace_base_url, serialize
- [x] T007 [P] Implement `src/parsers/toml.rs` — `TomlParser`: parse, find_base_url (recursive), replace_base_url, serialize
- [x] T008 [P] Implement `src/parsers/env.rs` — `EnvParser`: parse, find_base_url (line scan), replace_base_url, serialize (preserve comments)
- [x] T009 [P] Implement `src/backup.rs` — `create_backup(file_path)`, `rollback(backup_path, original_path)`, `restore_permissions()`
- [x] T010 Implement `src/writer.rs` — `safe_write()`: backup → parse → replace → serialize → write → rollback on failure
- [x] T011 [P] Implement `src/routes.rs` — `PresetRoute`, `CustomRoute`, `UserConfig` structs, `load_user_config()`, `save_user_config()`, `add/edit/delete_custom_route()`
- [x] T012 [P] Implement `src/ui.rs` — `print_tool_cards()`, `print_success()`, `print_error()`, `print_warning()`, color constants

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 — 查看三工具当前线路状态 (Priority: P1) 🎯 MVP

**Goal**: 用户打开工具后看到三个工具卡片的当前线路地址和状态

**Independent Test**: 运行 `cargo run`，确认三个卡片显示，已安装工具显示当前 base_url

- [x] T013 [US1] Implement Node.js/npm detection in `main.rs` — `check_node_installed()`, `check_npm_installed()`
- [x] T014 [US1] Implement tool status detection in `main.rs` — `detect_tool_status(tool)` returns config exists, base_url value, parseable
- [x] T015 [US1] Implement main loop in `main.rs` — display cards, show action menu, handle selection

**Checkpoint**: User Story 1 functional — three tool cards display with status

---

## Phase 4: User Story 2 — 切换单个工具的 API 线路 (Priority: P1) 🎯 MVP

**Goal**: 用户选择线路并确认后，配置文件中 base_url 被更新

**Independent Test**: 选择某工具的线路并切换，验证配置文件中 base_url 已更新

- [x] T016 [US2] Implement route selection menu in `main.rs` — list preset + custom routes, return selected route URL
- [x] T017 [US2] Implement switch confirmation prompt in `main.rs` — show tool name, route name, target URL, confirm/cancel
- [x] T018 [US2] Implement switch execution in `main.rs` — call `safe_write()`, handle base_url-not-found warning, display result
- [x] T019 [US2] Implement status refresh after switch — re-detect and re-display tool cards

**Checkpoint**: User Stories 1 & 2 functional — MVP complete

---

## Phase 5: User Story 3 — 管理自定义线路 (Priority: P2)

**Goal**: 用户新增/编辑/删除自定义线路，持久化到 config.json

**Independent Test**: 新增一条自定义线路，重启工具后仍然存在

- [x] T020 [US3] Implement custom route management menu in `main.rs` — list/add/edit/delete options
- [x] T021 [US3] Implement add custom route flow — name input + URL input + validation + save
- [x] T022 [US3] Implement edit custom route flow — select route + edit name/URL + validation + save
- [x] T023 [US3] Implement delete custom route flow — select route + confirm + delete

**Checkpoint**: User Stories 1-3 functional

---

## Phase 6: User Story 4 — 安装未检测到的 CLI 工具 (Priority: P2)

**Goal**: 配置文件不存在时提供安装按钮，安装后初始化配置

**Independent Test**: 在未安装某工具时点击安装，验证工具被安装且配置文件被创建

- [x] T024 [US4] Implement install command execution in `main.rs` — run install command, capture output, handle failure
- [x] T025 [US4] Implement default config initialization — write default config with selected route's base_url after install
- [x] T026 [US4] Implement permission handling — if install fails, display full command for manual execution

**Checkpoint**: User Stories 1-4 functional

---

## Phase 7: User Story 5 — 打开配置目录和文件 (Priority: P3)

**Goal**: 用户可打开配置目录或配置文件

**Independent Test**: 点击「打开目录」，验证系统文件管理器打开了对应目录

- [x] T027 [US5] Implement open directory in `main.rs` — `open_path_in_explorer(path)` cross-platform
- [x] T028 [US5] Implement open file in `main.rs` — `open_file_in_editor(path)` cross-platform
- [x] T029 [US5] Implement directory-not-exists handling — prompt user to create

**Checkpoint**: All user stories functional

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T030 Run `cargo build` — 0 errors, 0 warnings, 36 tests pass
- [x] T031 Run `cargo clippy` — 0 warnings
- [ ] T032 Manual end-to-end testing on Windows
- [x] T033 Create `.gitignore` for Rust project

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 (P1) → US2 (P1) → US3 (P2) → US4 (P2) → US5 (P3)
- **Polish (Phase 8)**: Depends on all user stories being complete

### Parallel Opportunities

- T005-T009 (parsers, backup, routes, ui) can all be implemented in parallel
- T027-T029 (open directory/file) can be implemented in parallel

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1 (display status)
4. Complete Phase 4: US2 (switch routes)
5. **STOP and VALIDATE**: Test MVP independently

### Incremental Delivery

6. Add US3 (custom routes) → Test
7. Add US4 (install tools) → Test
8. Add US5 (open config) → Test
9. Polish phase → Final validation
