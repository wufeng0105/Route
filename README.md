# Route — API 线路切换工具

跨平台桌面应用，管理 Codex CLI、Claude Code、Gemini CLI 的 API 线路切换。

基于 Tauri 2 + Rust 构建，支持 Windows、macOS、Linux 三平台。

## 功能

- **线路切换** — 一键切换三个 CLI 工具的 API base_url，写入前自动备份，失败自动回滚
- **预设线路** — 全球高保、国内优化等预设线路，开箱即用
- **自定义线路** — 按工具独立配置自定义中转站 URL
- **环境检测** — 自动检测 Node.js/npm 安装状态
- **工具安装** — 内置安装命令，安装后自动写入默认配置

## 支持的工具

| 工具 | 配置路径 | 格式 |
|------|---------|------|
| Codex CLI | `~/.codex/config.toml` | TOML |
| Claude Code | `~/.claude/settings.json` | JSON |
| Gemini CLI | `~/.gemini/.env` | ENV |

## 前置条件

### 全平台

- [Rust](https://rustup.rs/) stable (≥1.77)
- [Tauri CLI 2](https://v2.tauri.app/) — `cargo install tauri-cli --version "^2"`

### Windows 额外

- [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（含 MSVC C++ 工作负载）
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)（Windows 11 预装）
- 项目使用 `rust-toolchain.toml` 自动指定 MSVC 工具链（静态链接 WebView2Loader，产出单文件 exe）

### macOS 额外

- Xcode Command Line Tools — `xcode-select --install`

### Linux 额外

- WebKitGTK — `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file`

## 构建

```bash
# 克隆项目
git clone <repo-url>
cd Route

# Debug 编译
cargo build

# 发布构建（产出单文件 exe）
Cd src-tauri; npx tauri build
```

Windows 产出位于 `target/release/bundle/nsis/route_0.1.0_x64-setup.exe`（安装包）和 `target/release/app.exe`（单文件绿色版）。

### Windows 构建

```powershell
cd src-tauri
.\build.ps1 dev      # 开发模式
.\build.ps1 build    # Debug 编译
.\build.ps1 release  # 发布构建（单文件 exe）
```

> 使用 MSVC 工具链，`WebView2Loader` 静态链接到 exe 内部，无需分发 DLL。

## 测试

```bash
# 运行核心逻辑测试（无需 Tauri 运行时）
cargo test -p route-core

# 运行集成测试
cargo test -p route-core --test integration_tests
```

## 项目结构

```
Route/
├── crates/core/         # 核心逻辑库（无 Tauri 依赖，可独立测试）
├── src-tauri/           # Tauri 应用层（commands 薄层）
├── frontend/            # 前端静态文件（HTML + JS + Tailwind）
├── tests/               # 集成测试
└── specs/               # 规格文档（SPC 流程）
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 后端 | Rust (MSVC 工具链) |
| 前端 | 原生 HTML + Tailwind CSS + JavaScript（资源本地化，无 CDN 依赖） |
| 字体 | Inter + JetBrains Mono（内嵌） |
| 图标 | Material Symbols Outlined（内嵌） |
| CI | GitHub Actions（三平台） |

## 开发流程

本项目遵循 SPC（Specification-Planning-Construction）流程。详见 [CLAUDE.md](./CLAUDE.md)。

## License

MIT
