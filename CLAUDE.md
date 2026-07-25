# Route — Tauri 跨平台桌面应用项目规范

> 本文件替代所有历史文档（含 CLI 版本规范），作为项目唯一权威开发规范。
> Constitution v3.0.0 已同步更新，反映 Tauri 桌面应用架构。

---

## 第一部分：代码库现状分析

### 1.1 项目当前状态

项目已完成 Cargo workspace 拆分，核心逻辑独立为 `crates/core/` library crate，Tauri 应用层为薄层 commands。当前项目结构：

```
Route/
├── Cargo.toml                    # Workspace 根配置
├── Cargo.lock                    # 依赖锁定
├── rust-toolchain.toml           # 锁定 MSVC 工具链（单文件 exe）
├── CLAUDE.md                     # 本文件（项目规范）
├── README.md                     # 项目说明、构建指南
├── .gitignore                    # Git 忽略规则
├── crates/
│   └── core/                     # 核心逻辑库（无 Tauri 依赖）
│       ├── Cargo.toml            # 依赖: serde, serde_json, toml, dirs, regex
│       ├── src/
│       │   ├── lib.rs            # 公开 API（pub mod 重新导出）
│       │   ├── config_paths.rs   # 跨平台路径解析
│       │   ├── parsers/          # 配置文件解析器
│       │   │   ├── mod.rs        # ConfigFormat + ParsedConfig 统一接口
│       │   │   ├── toml.rs      # TOML 测试
│       │   │   ├── json.rs      # JSON 测试
│       │   │   └── env.rs        # ENV 测试
│       │   ├── backup.rs         # 备份与回滚
│       │   ├── writer.rs         # 安全写入（safe_write + write_default_config）
│       │   ├── routes.rs        # 线路管理（预设 + 自定义 CRUD）
│       │   └── tools.json        # 工具定义（JSON 驱动）
│       └── tests/
│           └── integration_tests.rs  # 集成测试（23 个）
├── src-tauri/                    # Tauri 应用层（依赖 core）
│   ├── Cargo.toml                # 依赖: tauri, tauri-plugin-log, route-core
│   ├── tauri.conf.json          # identifier: com.route.app
│   ├── .cargo/config.toml       # MSVC 工具链配置
│   ├── build.ps1                 # Windows 构建辅助（MSVC，自动检测 VS Build Tools）
│   ├── build.rs                 # Tauri 构建脚本
│   ├── capabilities/            # Tauri 权限配置
│   ├── icons/                   # 应用图标
│   └── src/
│       ├── main.rs              # Tauri 入口
│       └── lib.rs               # Tauri commands 薄层（调用 route_core::）
├── frontend/                    # 前端静态文件
│   ├── index.html               # 主界面 HTML
│   ├── app.js                   # 前端逻辑（调用 Tauri commands）
│   └── assets/                  # 本地化资源（无 CDN 依赖）
│       ├── tailwind.js           # Tailwind CSS（407KB）
│       ├── fonts.css             # Inter + JetBrains Mono 字体定义
│       ├── icons.css             # Material Symbols Outlined 图标定义
│       └── fonts/                # woff2 字体文件（14 个）
├── specs/                        # 规格文档
│   ├── 002-tauri-rebuild/       # 当前重构建规格
│   │   ├── spec.md
│   │   ├── plan.md
│   │   └── tasks.md
│   └── archive/                 # 归档的旧规格
│       └── 001-api-line-switcher/
├── .github/workflows/           # CI 配置
└── .specify/                    # Speckit 工具链
    └── memory/constitution.md   # Constitution v3.0.0
```

### 1.2 问题清单与修复状态

共 30 项问题，已修复 21 项，剩余 9 项。

#### P0 — 阻塞性问题（全部已修复 ✅）

| # | 问题 | 状态 | 修复方式 |
|---|------|------|---------|
| 1 | CI 工作流失效 | ✅ 已修复 | 工作流指向 workspace，`cargo test -p route-core` + `cargo build -p app` |
| 2 | Tauri 测试无法运行（DLL 依赖） | ✅ 已解决 | 拆分 `crates/core/`，core crate 无 Tauri 依赖 |
| 3 | 无集成测试 | ✅ 已建立 | 23 个集成测试覆盖 safe_write 全格式、备份回滚、线路 CRUD |
| 4 | Constitution 过期 | ✅ 已更新 | 升级至 v3.0.0，反映 Tauri 架构 |

#### P1 — 代码缺陷（大部分已修复）

| # | 问题 | 状态 | 详情 |
|---|------|------|------|
| 5 | `is_valid_url` 运算符优先级 Bug | ✅ 已修复 | 改为 `(a \|\| b) && c`，添加回归测试 |
| 6 | `write_default_config` 格式不一致 | ✅ 已修复 | TOML/ENV 统一用正则替换，与 `safe_write` 一致 |
| 7 | `openAddRoute()` 缺少 toolId | ✅ 已修复 | 新增 `openAddRouteFromManage()` 弹出工具选择 |
| 8 | `install_tool` async 但无 await | ✅ 已修复 | 移除 `async` 标记 |
| 9 | 前端 `escape()` 不转义单引号 | ✅ 已修复 | 添加 `'` → `&#39;` |
| 10 | `handleInstall` 使用 `prompt()` | ✅ 已修复 | 替换为自定义 modal |

#### P2 — 架构与设计问题（大部分已修复）

| # | 问题 | 状态 | 详情 |
|---|------|------|------|
| 11 | 核心逻辑与 Tauri 耦合 | ✅ 已拆分 | `crates/core/` 独立 library crate |
| 12 | `tauri.conf.json` identifier 默认值 | ✅ 已修复 | 改为 `com.route.app` |
| 13 | `build.ps1` 硬编码用户路径 | ✅ 已修复 | 切换到 MSVC 工具链，不再需要 MinGW |
| 14 | 前端 CDN 依赖 | ✅ 已修复 | Tailwind/字体/图标已本地化到 `frontend/assets/` |
| 15 | "管理线路"弹窗描述误导 | ✅ 已修复 | 改为"每个工具独立配置" |
| 16 | Stitch 设计稿未集成 | ✅ 已清理 | 4 个 stitch_*.html 已删除 |
| 17 | `specs/` 含过期文档 | ✅ 已归档 | 移至 `specs/archive/` |
| 18 | 无 `.gitignore` | ✅ 已创建 | 忽略 `target/`、`*.log` 等 |
| 19 | 无 `README.md` | ✅ 已创建 | 含项目说明、构建指南 |

#### P3 — 代码质量（待处理）

| # | 问题 | 状态 | 详情 |
|---|------|------|------|
| 20 | Dead code 警告 | ⬜ 待清理 | `tool_config_exists`、`get_route_url_for_tool` 未被调用 |
| 21 | `SwitchResult` 多余 `#[allow(dead_code)]` | ⬜ 待清理 | 所有字段实际都有使用 |
| 22 | `toolId` 非 snake_case | ⬜ 保留 | Tauri 命令参数命名约定，可加 `#[allow]` |
| 23 | `backup.rs` 时间戳用 UTC | ⬜ 待评估 | 可引入 `chrono` crate 或保持自实现 |
| 24 | ENV 序列化丢失尾部换行 | ⬜ 待修复 | `serialize()` 的 `pop()` 逻辑需调整 |
| 25 | JSON 序列化固定 2 空格缩进 | ⬜ 可接受 | Claude Code 实际用 2 空格 |
| 26 | TOML 序列化可能重排键序 | ✅ 已规避 | `safe_write` 用正则替换而非 parse-serialize |
| 27 | `load_user_config` 静默吞错 | ⬜ 待改进 | 解析失败应返回警告 |
| 28 | 前端 Mock 数据无提示 | ⬜ 待修复 | 应显式提示后端不可用 |
| 29 | `openAuthFile` 硬编码 | ⬜ 待修复 | 应从 tools.json 配置驱动 |
| 30 | 编辑线路复用"添加"弹窗脆弱 | ⬜ 待修复 | 应用独立编辑弹窗 |

### 1.3 测试覆盖现状

| 模块 | 单元测试 | 集成测试 | 运行状态 |
|------|---------|---------|---------|
| `backup.rs` | 4 个 | 2 个（备份+回滚完整流程、时间戳格式） | ✅ 全部通过 |
| `config_paths.rs` | 3 个 | 2 个（路径解析、全工具路径） | ✅ 全部通过 |
| `parsers/mod.rs` | 8 个 | 3 个（TOML/JSON/ENV round-trip） | ✅ 全部通过 |
| `parsers/toml.rs` | 5 个 | — | ✅ 全部通过 |
| `parsers/json.rs` | 6 个 | — | ✅ 全部通过 |
| `parsers/env.rs` | 6 个 | — | ✅ 全部通过 |
| `routes.rs` | 4 个 | 7 个（CRUD、校验、工具定义、预设线路） | ✅ 全部通过 |
| `writer.rs` | 2 个 | 5 个（safe_write 全格式、备份内容、默认配置） | ✅ 全部通过 |
| `lib.rs` (Tauri commands) | 无 | 无 | N/A（薄层，依赖 core） |
| **合计** | **38 个** | **23 个** | **全部通过（61 个测试）** |

**测试命令**：
```bash
cargo test -p route-core              # 全部测试（单元 + 集成）
cargo test -p route-core --lib        # 仅单元测试
cargo test -p route-core --test integration_tests  # 仅集成测试
cargo build -p app                    # 编译 Tauri 应用
```

---

## 第二部分：SPC 开发流程规范

### 2.1 SPC 定义

**SPC = Specification - Planning - Construction**（规格-规划-构建），是本项目强制执行的三阶段开发流程：

```
┌─────────────────────────────────────────────────────────────┐
│                    SPC 开发流程                              │
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐   │
│  │  S: 规格     │ →  │  P: 规划     │ →  │  C: 构建     │   │
│  │  Specification│    │  Planning    │    │  Construction│   │
│  └──────────────┘    └──────────────┘    └──────────────┘   │
│         │                   │                   │            │
│         ▼                   ▼                   ▼            │
│    需求规格文档         实施计划文档         代码+测试+文档    │
│    spec.md             plan.md              可交付产物       │
│                                                             │
│  质量门：每个阶段结束时必须通过审查才能进入下一阶段           │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 S 阶段：Specification（规格）

**产出**：`specs/<feature-id>/spec.md`

**步骤**：

1. **需求收集** — 明确功能目标、用户故事、边界条件
2. **约束识别** — 标注非功能性约束（性能、安全、跨平台）
3. **规格撰写** — 按以下结构编写 `spec.md`：
   - 功能概述
   - 用户故事（As a... I want... so that...）
   - 功能需求（MUST / SHOULD / MAY）
   - 非功能需求
   - 约束与假设
   - 验收标准（可测试的 Given-When-Then）
4. **规格审查** — 检查完整性、可测试性、无歧义

**质量门**：
- [ ] 所有 MUST 需求有对应验收标准
- [ ] 验收标准使用 Given-When-Then 格式
- [ ] 无未解决的歧义
- [ ] 与 Constitution 不冲突

### 2.3 P 阶段：Planning（规划）

**产出**：`specs/<feature-id>/plan.md` + `specs/<feature-id>/tasks.md`

**步骤**：

1. **技术方案设计** — 架构、模块划分、数据流、错误处理策略
2. **依赖分析** — 外部 crate 选择、版本锁定、许可证检查
3. **任务分解** — 按 SMART 原则拆分为可独立执行的任务
4. **任务排序** — 按依赖关系拓扑排序
5. **风险评估** — 标注高风险任务及缓解措施

**`plan.md` 结构**：
- 技术架构（含模块图）
- 数据模型
- 错误处理策略
- 测试策略
- 依赖清单

**`tasks.md` 结构**：
- 编号任务列表（含优先级、依赖关系、预估工时）
- 每个任务标注：涉及文件、验收条件

**质量门**：
- [ ] 所有 spec 中的需求有对应任务
- [ ] 任务间依赖关系明确无循环
- [ ] 高风险任务有缓解方案
- [ ] 测试策略覆盖所有验收标准

### 2.4 C 阶段：Construction（构建）

**产出**：代码 + 测试 + 文档更新

**步骤**：

1. **TDD 循环** — 对每个任务执行 Red-Green-Refactor：
   - Red：先写失败的测试
   - Green：写最小实现使测试通过
   - Refactor：重构代码，测试仍通过
2. **代码审查** — 完成后执行代码审查
3. **集成验证** — 所有任务完成后执行集成测试
4. **文档同步** — 更新 CLAUDE.md、README 等文档

**质量门**：
- [ ] 所有测试通过（`cargo test`）
- [ ] Clippy 无警告（`cargo clippy -- -D warnings`）
- [ ] 代码格式化（`cargo fmt --check`）
- [ ] CI 三平台通过（Ubuntu / macOS / Windows）
- [ ] 文档与代码一致

### 2.5 SPC 执行规则

1. **不跳阶段** — 不可在 spec 未审查通过时开始编码
2. **不跳质量门** — 不可在测试未通过时标记任务完成
3. **文档先行** — 任何代码变更必须有对应规格文档
4. **可追溯** — 每个 PR 可追溯到 spec → plan → task → code
5. **Constitution 优先** — SPC 流程受 Constitution 约束，冲突时以 Constitution 为准

---

## 第三部分：项目重规划

### 3.1 重构建进度

当前重构建（feature `002-tauri-rebuild`）进度：

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 1: Setup | ✅ 完成 | `.gitignore`、归档旧 specs、README.md |
| Phase 2: Workspace 拆分 | ✅ 完成 | Cargo workspace、`crates/core/`、模块迁移、Tauri 配置修正 |
| Phase 3: CI 修复 | ✅ 完成 | GitHub Actions 指向 workspace，分别测试 core 和构建 app |
| Phase 4: 缺陷修复 | ✅ 部分完成 | is_valid_url ✅、write_default_config ✅、escape() ✅、openAddRoute ✅；install_tool async ⬜、prompt() ⬜ |
| Phase 5: 测试体系 | ✅ 完成 | 23 个集成测试 + Constitution v3.0.0 |
| Phase 6: 文档体系 | ✅ 完成 | README ✅、spec/plan/tasks ✅、归档旧 specs ✅、stitch 清理 ✅ |
| Phase 7: 前端健壮性 | ⬜ 待执行 | CDN 本地化、编辑弹窗独立、Mock 提示 |

### 3.2 剩余任务

#### P1 缺陷（2 项）

| 任务 | 详情 | 位置 |
|------|------|------|
| 修复 `install_tool` async | 移除 `async` 或改用 `spawn_blocking` | `src-tauri/src/lib.rs:274` |
| 替换 `prompt()` 为自定义弹窗 | 安装线路选择用 Tauri dialog 或自定义 modal | `frontend/app.js:426` |

#### P2 前端健壮性（4 项）

| 任务 | 详情 | 位置 |
|------|------|------|
| 本地化 CDN 资源 | Tailwind CSS、字体、Material Symbols 改为本地文件 | `frontend/index.html:7-9` |
| 修复编辑线路弹窗 | 用独立编辑弹窗而非复用添加弹窗 | `frontend/app.js:352-378` |
| Tauri API 不可用时显式提示 | 不再静默降级到 Mock | `frontend/app.js:14-39` |
| 修复 `openAuthFile` 硬编码 | 从 tools.json 配置驱动 auth 文件名 | `frontend/app.js:401` |

#### P3 代码质量（5 项）

| 任务 | 详情 |
|------|------|
| 清理 dead code | 移除 `tool_config_exists`、`get_route_url_for_tool` 或添加 `#[allow]` |
| 修复 ENV 序列化尾部换行 | `parsers/mod.rs` serialize() 的 `pop()` 逻辑 |
| `load_user_config` 解析失败返回警告 | 静默回退应改为显式提示 |
| 添加 `#[allow(non_snake_case)]` | `lib.rs` 中 `toolId` 参数 |

---

## 第四部分：技术栈与约束

### 4.1 技术栈

| 层级 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 桌面框架 | Tauri 2 | 2.11+ | 跨平台桌面应用壳 |
| 后端语言 | Rust | stable (≥1.77) | 核心逻辑 |
| 前端 | 原生 HTML + Tailwind CSS + JS | — | 静态前端，无构建工具 |
| 字体 | Inter + JetBrains Mono | — | UI 字体 + 代码字体 |
| 图标 | Material Symbols Outlined | — | UI 图标 |
| CI | GitHub Actions | — | 三平台自动化 |
| 规格工具 | Speckit | — | SPC 流程辅助 |

### 4.2 支持的工具与配置路径

| 工具 | 配置目录 | 配置文件 | 格式 | base_url 字段名 | 安装命令 |
|------|---------|---------|------|----------------|---------|
| Codex CLI | `~/.codex/` | `config.toml` | TOML | `base_url` | `npm install -g @openai/codex` |
| Claude Code | `~/.claude/` | `settings.json` | JSON | `ANTHROPIC_BASE_URL` | Win: `irm https://claude.ai/install.ps1 \| iex`，Unix: `curl -fsSL https://claude.ai/install.sh \| bash` |
| Gemini CLI | `~/.gemini/` | `.env` | ENV | `GOOGLE_GEMINI_BASE_URL` | `npm install -g @google/gemini-cli` |

### 4.3 跨平台规则

- 所有路径使用 `dirs::home_dir()` + `PathBuf::join()`，不手动拼接分隔符
- 文件权限：Linux/macOS 写入前读取并保留原权限，写入后恢复
- 文件名：使用官方精确小写文件名
- 行尾符：统一 `\n`（LF）
- 编码：UTF-8 无 BOM
- 打开路径：Windows → `explorer`，macOS → `open`，Linux → `xdg-open`
- npm 检测：Windows 用 `npm.cmd`，回退 `cmd /c npm`；Unix 直接 `npm`

### 4.4 安全写入规则

1. 写入前：复制原文件为 `<filename>.backup.<timestamp>`
2. 写入失败：删除新文件，重命名备份为原文件名（回滚）
3. 成功后：保留备份文件
4. 权限不足：提示用户手动检查，不静默失败

### 4.5 配置文件解析规则

- **JSON**（Claude Code）：`serde_json` 解析-序列化，递归搜索 `base_url`
- **TOML**（Codex CLI）：正则表达式替换（保留原始格式），读取时用正则提取
- **ENV**（Gemini CLI）：正则表达式替换（保留原始格式），读取时用正则提取
- `base_url` 搜索不区分大小写，递归遍历所有层级
- 未找到 `base_url` 字段时发出警告，允许用户确认是否继续

### 4.6 数据模型

**用户配置**（`~/.api-line-switcher/config.json`）：

```json
{
  "presetRoutes": [
    {
      "id": "global",
      "name": "全球高保",
      "urls": {
        "codex": "https://...",
        "claude": "https://...",
        "gemini": "https://..."
      }
    }
  ],
  "customRoutes": [
    {
      "toolId": "codex",
      "name": "我的中转站",
      "url": "https://api.example.com"
    }
  ]
}
```

- 预设线路始终从代码内 `default_user_config()` 加载（不信任用户文件中的预设值）
- 自定义线路按工具独立配置（每个 `CustomRoute` 绑定一个 `toolId`）

### 4.7 开发原则

1. 不硬编码配置字段路径，自动搜索 `base_url`
2. 不读取/存储配置文件中的敏感信息（如 API Key），仅修改 URL
3. 写入前必须备份
4. 跨平台路径必须使用 `PathBuf::join()` + `dirs::home_dir()`
5. 所有用户操作需有明确反馈（成功/失败/警告）
6. 工具定义和预设线路通过 JSON 配置，不写死在代码中
7. 核心逻辑与 Tauri 框架解耦，可独立测试
