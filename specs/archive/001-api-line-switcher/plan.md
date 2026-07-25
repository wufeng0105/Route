# Implementation Plan: CLI API Line Switcher

**Branch**: `001-api-line-switcher` | **Date**: 2026-07-24 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-api-line-switcher/spec.md`

## Summary

管理 Codex CLI、Claude Code、Gemini CLI 三个终端工具的 API 线路切换。用户通过交互式终端 UI 查看三工具当前线路状态、一键切换预设/自定义线路、管理自定义线路、安装未检测到的工具、打开配置目录/文件。使用 Rust 编译为单二进制文件，零运行时依赖。

## Technical Context

**Language/Version**: Rust 1.97 (stable)

**Primary Dependencies**:
- `serde` + `serde_json` — JSON 序列化/反序列化（Claude Code 配置、用户配置、tools.json）
- `toml` — TOML 解析（Codex CLI 配置）
- `dirs` — 跨平台 Home 目录获取
- `inquire` — 交互式终端 UI（选择菜单、确认提示、文本输入）
- `colored` — 终端彩色输出

**Storage**: 文件系统
- 用户配置：`~/.api-line-switcher/config.json`（预设线路 + 自定义线路）
- 工具定义：`src/tools.json`（编译时嵌入二进制，`include_str!`）
- 目标配置文件：`~/.codex/config.toml`、`~/.claude/settings.json`、`~/.gemini/.env`

**Testing**: `cargo test`（单元测试 + 集成测试）

**Target Platform**: Windows 11 / Linux / macOS（跨平台）

**Project Type**: CLI 工具（单二进制文件）

**Performance Goals**: 配置文件读取+修改+写入（含备份）≤ 2 秒

**Constraints**:
- 编译产物 ≤ 20MB
- 零运行时依赖（用户无需安装任何运行时）
- 仅修改 `base_url` 字段，不触碰敏感信息

**Scale/Scope**: 小规模工具，<100 条线路配置，单用户单机使用

## Constitution Check

| 原则 | 状态 | 说明 |
|------|------|------|
| I. 跨平台路径优先 | ✅ 通过 | 使用 `dirs::home_dir()` + `PathBuf::join()` |
| II. 安全写入 | ✅ 通过 | 备份→写入→失败回滚流程 |
| III. 精确解析 | ✅ 通过 | `serde_json`/`toml` crate + 自实现 ENV 解析 |
| IV. 最小依赖与最小体积 | ✅ 通过 | 6 个 crate，编译为单二进制 |
| V. URL-Only 修改 | ✅ 通过 | 仅搜索替换 `base_url` 字段 |
| VI. 用户反馈先行 | ✅ 通过 | 所有操作有彩色反馈，切换需确认 |

## Project Structure

### Documentation (this feature)

```text
specs/001-api-line-switcher/
├── plan.md              # 本文件
├── spec.md              # 功能规范
├── checklists/
│   └── requirements.md  # 需求检查清单
└── tasks.md             # 任务清单
```

### Source Code (repository root)

```text
Route/
├── Cargo.toml               # Rust 项目配置
├── CLAUDE.md                # 项目规范
├── src/
│   ├── main.rs              # 入口 + 终端 UI + 主逻辑编排
│   ├── config_paths.rs      # 跨平台路径解析
│   ├── parsers/             # 配置文件解析器
│   │   ├── mod.rs           # Parser trait + 统一接口
│   │   ├── toml.rs          # TOML 解析（Codex）
│   │   ├── json.rs          # JSON 解析（Claude）
│   │   └── env.rs           # ENV 解析（Gemini）
│   ├── backup.rs            # 备份与回滚
│   ├── writer.rs            # 安全写入（协调备份+解析+写入+回滚）
│   ├── routes.rs            # 线路管理（预设 + 自定义 + 持久化）
│   ├── tools.json           # 工具定义（编译时嵌入）
│   └── ui.rs                # 终端 UI 渲染（卡片布局 + 彩色输出）
├── tests/                   # 集成测试
│   ├── parsers.rs           # 各格式解析与写入测试
│   ├── backup.rs            # 备份回滚测试
│   └── routes.rs            # 线路管理测试
└── .github/
    └── workflows/
        └── test.yml         # 跨平台 CI
```

**Structure Decision**: 单项目结构（CLI 工具），`src/` 下按功能模块组织。解析器使用子目录 `parsers/` 以隔离不同格式的实现。UI 渲染独立为 `ui.rs` 模块以便复用和测试。

## Key Design Decisions

### 1. base_url 递归搜索

对 JSON 和 TOML 格式，递归遍历 `serde_json::Value` / `toml::Value` 的所有层级，匹配包含 `base_url`（不区分大小写）的字段名。对 ENV 格式，逐行扫描键名。

### 2. tools.json 嵌入

使用 `include_str!("tools.json")` 在编译时嵌入工具定义，运行时无需读取外部文件。通过 `serde_json::from_str` 解析为结构体。

### 3. 终端 UI 卡片布局

三列卡片通过格式化文本实现：每列固定宽度，使用 Unicode 制表符绘制边框，`colored` crate 着色状态信息。交互菜单通过 `inquire::Select` 实现。

### 4. Node.js 检测

使用 `std::process::Command::new("node").arg("--version")` 检测 Node.js，`npm --version` 检测 npm。检测失败不阻断线路切换功能。

### 5. 配置文件格式保持

- JSON：`serde_json::to_string_pretty`（2 空格缩进）
- TOML：`toml::to_string`（标准格式）
- ENV：保持原文件注释和空行，仅替换匹配行的值

## Complexity Tracking

无宪法违规，无需记录复杂度例外。
