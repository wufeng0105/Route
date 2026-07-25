# Implementation Plan: Tauri 项目重构建

**Branch**: `002-tauri-rebuild` | **Date**: 2026-07-25 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-tauri-rebuild/spec.md`

## Summary

将 Route 项目从打补丁式开发重构为正式 SPC 流程管理的 Tauri 跨平台桌面应用。核心技术方案：拆分 Cargo workspace，将核心逻辑（parsers/backup/writer/routes/config_paths）独立为 `crates/core/` library crate（无 Tauri 依赖），Tauri 应用层变为薄层 commands。同时修复全部 P0/P1 缺陷，建立集成测试体系，更新文档。

## Technical Context

**Language/Version**: Rust stable (≥1.77)

**Primary Dependencies**:
- `serde` 1.0 + `serde_json` 1.0 — 序列化/反序列化
- `toml` 0.8 — TOML 解析
- `dirs` 5 — 跨平台 Home 目录
- `regex` 1 — 正则表达式（TOML/ENV 格式保留）
- `tauri` 2.11+ — 桌面应用壳（仅 src-tauri）
- `tauri-plugin-log` 2 — 日志（仅 src-tauri）

**Storage**: 文件系统（`~/.api-line-switcher/config.json` 用户配置，`~/.<工具>/` 工具配置）

**Testing**: `cargo test` — 单元测试（内嵌 `#[cfg(test)]`）+ 集成测试（`tests/` 目录）

**Target Platform**: Windows 11、macOS、Linux（三平台桌面）

**Project Type**: Cargo workspace（library + desktop-app）

**Performance Goals**: 启动 <2s，线路切换 <100ms，安装包 <20MB

**Constraints**: 离线可用（CDN 资源本地化），零运行时依赖（Tauri 用系统 WebView）

**Scale/Scope**: 3 个工具、2 条预设线路、<100 条自定义线路

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

当前 Constitution v2.0.0 与实际架构冲突：
- ❌ "UI：交互式终端（dialoguer crate）" → 实际为 Tauri WebView GUI
- ❌ "不使用 Tauri（终端工具无需 GUI 壳）" → 实际使用 Tauri
- ❌ "单二进制文件" → 实际为 Tauri 应用包

**修复方案**：Constitution 升级至 v3.0.0，在 US4 中完成。SPE C 流程受新 Constitution 约束。

## Project Structure

### Documentation (this feature)

```text
specs/002-tauri-rebuild/
├── spec.md              # 规格文档（S 阶段产出）
├── plan.md              # 本文件（P 阶段产出）
└── tasks.md             # 任务清单（P 阶段产出）
```

### Source Code (repository root)

```text
Route/
├── Cargo.toml                    # Workspace 根配置
├── CLAUDE.md                     # 项目规范
├── README.md                     # 项目说明（US4 新建）
├── .gitignore                    # Git 忽略规则（US4 新建）
├── crates/
│   └── core/                     # 核心逻辑库（无 Tauri 依赖）
│       ├── Cargo.toml            # 依赖: serde, serde_json, toml, dirs, regex
│       └── src/
│           ├── lib.rs            # 公开 API（pub use 重新导出）
│           ├── config_paths.rs   # 跨平台路径解析
│           ├── parsers/
│           │   ├── mod.rs       # ConfigFormat + ParsedConfig 统一接口
│           │   ├── toml.rs      # TOML 测试
│           │   ├── json.rs      # JSON 测试
│           │   └── env.rs       # ENV 测试
│           ├── backup.rs         # 备份与回滚
│           ├── writer.rs         # 安全写入（safe_write + write_default_config）
│           ├── routes.rs         # 线路管理（预设 + 自定义 CRUD）
│           └── tools.json        # 工具定义（JSON 驱动）
├── src-tauri/                    # Tauri 应用层（依赖 core）
│   ├── Cargo.toml                # 依赖: tauri, tauri-plugin-log, route-core
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── build.ps1                 # Windows 构建辅助（修复硬编码路径）
│   ├── capabilities/
│   ├── icons/
│   └── src/
│       ├── main.rs               # Tauri 入口
│       └── lib.rs                # Tauri commands 薄层（调用 core::）
├── frontend/                     # 前端静态文件
│   ├── index.html
│   ├── app.js
│   └── assets/                   # 本地化资源（US5 新建）
│       ├── tailwind.css
│       ├── fonts/
│       └── icons/
├── tests/                        # 集成测试
│   └── integration_tests.rs      # 端到端测试（依赖 route-core）
├── .github/workflows/
│   └── test.yml                  # CI 工作流（修复为 workspace 模式）
├── .specify/
│   └── memory/constitution.md   # Constitution（升级至 v3.0.0）
└── specs/
    ├── 002-tauri-rebuild/        # 本特性文档
    └── archive/                  # 归档的旧规格文档
        └── 001-api-line-switcher/
```

**Structure Decision**: 选择 Cargo workspace 方案（而非单 crate + features），因为：
1. core crate 完全无 Tauri 依赖，测试二进制不链接 WebView2 DLL
2. 物理隔离防止核心逻辑意外引入 Tauri 依赖
3. CI 可分别 `cargo test -p route-core` 和 `cargo build -p app`

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      Cargo Workspace                            │
│                                                                 │
│  ┌─────────────────────────────────────────────────────┐        │
│  │              crates/core (route-core)                │        │
│  │                                                     │        │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │        │
│  │  │ config_paths │  │   parsers    │  │  backup   │ │        │
│  │  │              │  │ (toml/json/ │  │           │ │        │
│  │  │ home_dir()   │  │  env)       │  │ create+   │ │        │
│  │  │ PathBuf      │  │             │  │ rollback  │ │        │
│  │  └──────────────┘  └──────────────┘  └───────────┘ │        │
│  │                                               │     │        │
│  │  ┌──────────────┐  ┌──────────────┐          │     │        │
│  │  │   routes     │  │   writer     │◄─────────┘     │        │
│  │  │              │  │              │                │        │
│  │  │ load_tools   │  │ safe_write   │                │        │
│  │  │ add/edit/    │  │ write_default│                │        │
│  │  │ delete route │  │ read_base_url│                │        │
│  │  └──────────────┘  └──────────────┘                │        │
│  │         │                  │                       │        │
│  │         ▼                  ▼                       │        │
│  │     tools.json       ~/.<tool>/config              │        │
│  │     (embedded)       (user files)                  │        │
│  └─────────────────────────────────────────────────────┘        │
│                         │ pub use                               │
│                         ▼                                       │
│  ┌─────────────────────────────────────────────────────┐        │
│  │              src-tauri (app)                         │        │
│  │                                                     │        │
│  │  ┌──────────────────────────────────────────────┐   │        │
│  │  │ lib.rs — Tauri Commands 薄层                  │   │        │
│  │  │                                                │   │        │
│  │  │  get_tool_statuses()    → core::routes::load_tools()  │   │
│  │  │  get_user_config()      → core::routes::load_user_config() │
│  │  │  check_env()            → std::process::Command  │   │        │
│  │  │  switch_route()         → core::writer::safe_write() │   │        │
│  │  │  add_custom_route()     → core::routes::add_custom_route() │
│  │  │  edit_custom_route()     → core::routes::edit_custom_route() │
│  │  │  delete_custom_route()   → core::routes::delete_custom_route() │
│  │  │  open_config_dir()      → explorer/open/xdg-open │   │        │
│  │  │  open_config_file()     → explorer/open/xdg-open │   │        │
│  │  │  install_tool()         → Command + core::writer │   │        │
│  │  └──────────────────────────────────────────────┘   │        │
│  │                                                     │        │
│  │  ┌──────────┐                                       │        │
│  │  │ main.rs  │ → app_lib::run()                     │        │
│  │  └──────────┘                                       │        │
│  └─────────────────────────────────────────────────────┘        │
│                         │                                       │
│  ┌──────────────────────┼───────────────────────────┐          │
│  │            tests/ (integration)                    │          │
│  │                                                    │          │
│  │  integration_tests.rs → route_core::writer::safe_write()    │
│  │                       → route_core::backup::create_backup() │
│  │                       → route_core::routes::add_custom_route() │
│  └────────────────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────────┘

         Frontend (WebView)
         ┌───────────────────────────┐
         │  index.html + app.js       │
         │  invoke('switch_route') ───┼──→ Tauri command
         │  invoke('get_tool_statuses')┼──→ Tauri command
         └───────────────────────────┘
```

## Data Flow

### 线路切换流程

```
用户点击"切换"按钮
    │
    ▼
前端: handleSwitch(toolId, url, name)
    │  打开确认弹窗 → 用户确认
    ▼
invoke('switch_route', { toolId, targetUrl })
    │
    ▼
lib.rs: switch_route(tool_id, target_url)
    │  1. routes::load_tools() → 找到 tool 定义
    │  2. config_paths::get_tool_config_path() → 确定配置文件路径
    │  3. parsers::ConfigFormat::from_str() → 确定格式
    ▼
core::writer::safe_write(config_path, format, target_url, tool_name)
    │  4. 读取原文件内容
    │  5. 根据格式选择替换方式:
    │     - TOML → 正则替换（保留格式）
    │     - ENV → 正则替换（保留格式）
    │     - JSON → parse-serialize
    │  6. backup::create_backup() → 复制原文件为 .backup.<timestamp>
    │  7. 写入新内容
    │  8. 失败时 → backup::rollback() → 恢复原文件
    ▼
返回 SwitchResultDto → 前端 showToast
```

## Error Handling Strategy

| 错误类型 | 处理方式 | 用户反馈 |
|---------|---------|---------|
| Home 目录无法确定 | 返回 `None`/`Err` | Toast: "无法确定配置文件路径" |
| 配置文件不存在 | 返回 `config_exists: false` | UI 显示"未检测到"状态 |
| 配置文件解析失败 | 返回 `Err(错误信息)` | UI 显示"解析失败" + 错误详情 |
| base_url 字段未找到 | `base_url_found: false` | Toast 警告"未找到 base_url 字段" |
| 写入失败 | 自动回滚 + 返回错误 | Toast: "切换失败" + 错误原因 |
| 回滚也失败 | 返回双重错误 | Toast: "写入失败（回滚也失败）" |
| URL 校验失败 | 返回 `Err` | Toast: "URL 格式无效" |
| 线路名称重复 | 返回 `Err` | Toast: "线路名称已存在" |

## Test Strategy

### 测试分层

| 层级 | 范围 | 工具 | 运行命令 |
|------|------|------|---------|
| 单元测试 | 各模块独立功能 | `#[cfg(test)] mod tests` | `cargo test -p route-core --lib` |
| 集成测试 | 跨模块端到端流程 | `tests/integration_tests.rs` | `cargo test -p route-core --test integration_tests` |
| 编译验证 | Tauri 应用层编译 | `cargo build` | `cargo build -p app` |
| CI | 三平台全量 | GitHub Actions | push/PR 自动触发 |

### 测试覆盖矩阵

| 需求 | 单元测试 | 集成测试 |
|------|---------|---------|
| TOML parse/replace/serialize | ✅ 5 个 | ✅ safe_write TOML E2E |
| JSON parse/replace/serialize | ✅ 6 个 | ✅ safe_write JSON E2E |
| ENV parse/replace/serialize | ✅ 6 个 | ✅ safe_write ENV E2E |
| 备份创建 | ✅ 1 个 | ✅ 备份+回滚完整流程 |
| 回滚恢复 | ✅ 1 个 | ✅ 同上 |
| 路径解析 | ✅ 3 个 | ✅ 全工具路径解析 |
| 线路 CRUD | ✅ 4 个 | ✅ 增删改+校验 |
| URL 校验 | ✅ 1 个（需修复后） | ✅ 无效 URL 拒绝 |

## Dependency List

### crates/core/Cargo.toml

```toml
[package]
name = "route-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
dirs = "5"
regex = "1"
```

### src-tauri/Cargo.toml

```toml
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[lib]
name = "app_lib"
crate-type = ["staticlib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.6.3", features = [] }

[dependencies]
tauri = { version = "2.11.3", features = [] }
tauri-plugin-log = "2"
route-core = { path = "../crates/core" }
```

### Workspace Cargo.toml

```toml
[workspace]
members = ["crates/core", "src-tauri"]
resolver = "2"
```
