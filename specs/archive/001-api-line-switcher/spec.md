# Feature Specification: CLI API Line Switcher

**Feature Branch**: `001-api-line-switcher`

**Created**: 2026-07-24

**Status**: Draft

**Input**: User description: "管理 Codex CLI、Claude Code、Gemini CLI 的 API 线路切换的桌面工具"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 查看三工具当前线路状态 (Priority: P1)

用户打开工具后，一眼看到 Codex CLI、Claude Code、Gemini CLI 三个工具卡片，每个卡片显示该工具当前的 API 线路地址和状态（正常 / 未检测到配置）。

**Why this priority**: 用户需要先了解当前状态才能决定是否切换。这是所有后续操作的前提。

**Independent Test**: 打开工具，确认三个卡片都显示，且已安装工具的卡片显示当前 base_url 地址。

**Acceptance Scenarios**:

1. **Given** 三个 CLI 工具均已安装且配置文件存在，**When** 用户打开工具，**Then** 三个卡片各自显示当前的 base_url 值和「正常」状态
2. **Given** Codex CLI 未安装（配置文件不存在），**When** 用户打开工具，**Then** Codex 卡片显示「未检测到配置，可能未安装」状态，切换按钮禁用，但「打开目录」按钮可用
3. **Given** 某工具配置文件存在但无法解析（格式损坏），**When** 用户打开工具，**Then** 该卡片显示「配置文件解析失败」状态和错误详情

---

### User Story 2 - 切换单个工具的 API 线路 (Priority: P1)

用户在某个工具卡片上选择一条线路（全球高保 / 国内优化 / 自定义），点击切换后弹出确认弹窗，显示目标地址，确认后执行切换，成功后显示通知并更新当前状态。

**Why this priority**: 这是工具的核心价值——一键切换 API 线路。

**Independent Test**: 在任一已安装工具的卡片上选择线路并切换，验证配置文件中 base_url 已更新为目标地址。

**Acceptance Scenarios**:

1. **Given** Claude Code 已安装且配置正常，**When** 用户选择「全球高保」线路并确认切换，**Then** Claude Code 的 `settings.json` 中 base_url 字段被更新为 `https://api.aicodemirror.ai/api/claudecode`，卡片状态刷新为新地址
2. **Given** Codex CLI 已安装且配置正常，**When** 用户选择「国内优化」线路并确认切换，**Then** Codex 的 `config.toml` 中 base_url 字段被更新为 `https://api.claudecode.net.cn/api/codex/backend-api/codex`
3. **Given** Gemini CLI 已安装且配置正常，**When** 用户选择某自定义线路并确认切换，**Then** Gemini 的 `.env` 中 base_url 字段被更新为该自定义线路的 URL
4. **Given** 用户点击切换，**When** 确认弹窗出现，**Then** 弹窗显示目标工具名称、目标线路名称、目标地址，用户取消则不执行任何修改
5. **Given** 配置文件中未找到 base_url 字段，**When** 用户确认切换，**Then** 系统显示警告「未找到 base_url 字段」，允许用户确认是否继续

---

### User Story 3 - 管理自定义线路 (Priority: P2)

用户可以通过模态对话框新增、编辑、删除自定义线路。自定义线路保存后持久化到本地配置文件，下次打开工具时仍然可用。切换弹窗中可选择自定义线路。

**Why this priority**: 预设线路无法覆盖所有场景，用户需要添加自己的中转/代理线路。

**Independent Test**: 新增一条自定义线路，关闭并重新打开工具，确认该线路仍然存在且可被选用于切换。

**Acceptance Scenarios**:

1. **Given** 用户打开自定义线路管理对话框，**When** 用户填写名称「我的中转」和 URL `https://my-proxy.com/api` 并保存，**Then** 线路列表中新增该条目，且在三个工具的切换弹窗中均可选择该线路
2. **Given** 存在自定义线路「我的中转」，**When** 用户编辑其 URL 为 `https://new-proxy.com/api` 并保存，**Then** 线路列表更新，所有工具切换弹窗中该线路的地址同步更新
3. **Given** 存在自定义线路「我的中转」，**When** 用户删除该线路，**Then** 线路从列表和所有切换弹窗中移除
4. **Given** 用户尝试保存 URL 格式无效的自定义线路（如 `not-a-url`），**When** 点击保存，**Then** 系统拒绝保存并提示 URL 格式错误
5. **Given** 用户尝试保存名称为空的自定义线路，**When** 点击保存，**Then** 系统拒绝保存并提示名称不能为空

---

### User Story 4 - 安装未检测到的 CLI 工具 (Priority: P2)

当某工具的配置文件不存在时，卡片提供「安装」按钮。点击后系统执行该工具的安装命令，安装完成后使用默认配置（含选定线路的 base_url）初始化配置文件。若安装后已存在配置文件则先备份再写入。

**Why this priority**: 降低用户安装和配置门槛，发现没装某工具时可直接一键安装。

**Independent Test**: 在未安装某工具的状态下点击安装按钮，验证工具被安装且配置文件被创建并包含正确的 base_url。

**Acceptance Scenarios**:

1. **Given** Codex CLI 未安装（配置文件不存在），**When** 用户点击 Codex 卡片上的「安装」按钮，**Then** 系统执行 Codex CLI 安装命令，安装完成后创建默认配置文件并写入当前选定线路的 base_url
2. **Given** 安装命令执行完成且工具自动创建了配置文件，**When** 系统写入默认配置，**Then** 先将工具自动创建的配置文件备份为 `.backup.<timestamp>`，再写入含 base_url 的默认配置
3. **Given** 安装命令执行失败（如网络不通），**When** 安装过程结束，**Then** 系统显示安装失败通知和错误详情，不创建配置文件
4. **Given** 安装成功且配置文件已创建，**When** 流程完成，**Then** 卡片状态从「未检测到」更新为「正常」，显示新写入的 base_url，切换按钮变为可用

---

### User Story 5 - 打开配置目录和文件 (Priority: P3)

用户可以在每个工具卡片上点击「打开目录」按钮，系统调用文件管理器打开该工具的配置目录；点击「打开文件」按钮，用系统默认编辑器打开配置文件。

**Why this priority**: 便捷的辅助功能，用户可能需要手动查看或修改配置。

**Independent Test**: 点击「打开目录」按钮，验证系统文件管理器打开了对应的配置目录。

**Acceptance Scenarios**:

1. **Given** 某工具配置目录存在，**When** 用户点击「打开目录」按钮，**Then** 系统文件管理器打开该目录
2. **Given** 某工具配置文件存在，**When** 用户点击「打开文件」按钮，**Then** 系统默认编辑器打开该文件
3. **Given** 某工具配置目录不存在，**When** 用户点击「打开目录」按钮，**Then** 系统提示「目录不存在」并询问是否创建

---

### Edge Cases

- 配置文件中存在多个 base_url 字段（不同层级）时，全部替换为同一新值（一般配置文件中不会出现多个）
- 自定义线路的 URL 与预设线路重复时是否允许保存？
- 切换线路时配置文件被其他程序占用（如用户正在编辑），写入失败如何处理？
- 工具安装在非默认路径下（如自定义 Node.js prefix），配置文件位置是否不同？
- 同时打开多个工具实例，切换时是否有竞态问题？
- 安装命令需要较长时间，用户等待期间如何反馈进度？
- 配置文件中 base_url 字段存在但值为空字符串，是否视为有效？
- Node.js 已安装但版本过低（如低于 18），是否需要提示升级？

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 系统 MUST 在启动时检测三个 CLI 工具的配置文件是否存在，并显示对应状态
- **FR-002**: 系统 MUST 读取并显示每个已安装工具配置文件中当前的 base_url 值
- **FR-003**: 系统 MUST 提供两条预设线路：「全球高保」和「国内优化」，每条线路对三个工具有各自固定的 URL
- **FR-004**: 预设线路 URL MUST 为：
  - 全球高保：Claude Code → `https://api.aicodemirror.ai/api/claudecode`，Codex → `https://api.aicodemirror.ai/api/codex/backend-api/codex`，Gemini → `https://api.aicodemirror.ai/api/gemini`
  - 国内优化：Claude Code → `https://api.claudecode.net.cn/api/claudecode`，Codex → `https://api.claudecode.net.cn/api/codex/backend-api/codex`，Gemini → `https://api.claudecode.net.cn/api/gemini`
- **FR-005**: 用户 MUST 能为每个工具独立选择并切换线路
- **FR-006**: 切换操作 MUST 经确认弹窗（显示工具名称、线路名称、目标地址）后才执行
- **FR-007**: 系统 MUST 递归搜索配置文件中包含 `base_url`（不区分大小写）的字段名并替换其值
- **FR-008**: 系统 MUST 在写入配置文件前备份原文件，写入失败时自动回滚
- **FR-009**: 用户 MUST 能新增、编辑、删除自定义线路（名称 + URL）
- **FR-010**: 自定义线路 MUST 持久化存储，重启工具后仍然可用
- **FR-011**: 系统 MUST 校验自定义线路 URL 格式（合法 URL scheme + 非空）和名称非空
- **FR-012**: 当配置文件不存在时，系统 MUST 显示「未检测到配置，可能未安装」状态并禁用切换按钮
- **FR-013**: 当配置文件不存在时，系统 MUST 提供「安装」按钮，点击后执行对应 CLI 工具的安装命令
- **FR-014**: 安装完成后，系统 MUST 使用默认配置初始化配置文件；若配置文件已存在则先备份再写入
- **FR-015**: 用户 MUST 能通过「打开目录」按钮在系统文件管理器中打开配置目录
- **FR-016**: 用户 MUST 能通过「打开文件」按钮在系统默认编辑器中打开配置文件
- **FR-017**: 所有操作 MUST 有明确反馈（成功 / 失败 / 警告通知）
- **FR-018**: 系统 MUST 仅修改 base_url 字段值，不读取或修改其他敏感字段（如 API Key、Token）
- **FR-019**: 系统 MUST 在启动时检测运行环境是否已安装 Node.js 和 npm，并据此决定安装功能是否可用
- **FR-020**: 若 Node.js 或 npm 未检测到，系统 MUST 禁用安装按钮并显示提示（含 Node.js 官方安装链接），但线路切换功能（读写配置文件）仍 MUST 正常可用
- **FR-021**: 安装 CLI 工具时，若执行权限不足（如 Linux/macOS 的 npm 全局目录需要 sudo），系统 MUST 显示完整命令让用户在终端手动执行，不在应用内自动提权

### Key Entities *(include if feature involves data)*

- **预设线路 (PresetRoute)**：系统内置的固定线路，包含名称和对三个工具各自的 URL。不可编辑、不可删除。
- **自定义线路 (CustomRoute)**：用户创建的线路，包含名称和 URL（对三个工具通用同一 URL）。可新增、编辑、删除。持久化到 `~/.api-line-switcher/config.json`。
- **工具状态 (ToolStatus)**：每个 CLI 工具的当前状态，包括：是否检测到配置文件、当前 base_url 值、配置文件格式是否可解析。
- **切换记录 (SwitchResult)**：一次切换操作的结果，包括：目标工具、目标线路、目标地址、是否成功、备份文件路径、错误信息（如有）。

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 用户从打开工具到完成一次线路切换不超过 3 次点击（选择线路 → 确认 → 完成）
- **SC-002**: 系统能在 2 秒内完成配置文件的读取、修改和写入（含备份）
- **SC-003**: 三个 CLI 工具的线路切换互不影响，切换一个工具的线路不会改变其他工具的配置
- **SC-004**: 100% 的写入操作在失败时能自动回滚到原始状态，不损坏用户配置文件
- **SC-005**: 用户添加的自定义线路在工具重启后 100% 保留
- **SC-006**: 未安装某工具的用户能通过一键安装按钮完成安装和初始配置，无需手动查找安装命令
- **SC-007**: 安装包大小 MUST 在 20MB 以内，避免为简单工具引入过大的体积开销

## Assumptions

- 三个 CLI 工具的配置文件位置遵循 `~/.<工具名>/` 惯例，在所有平台上一致
- 三个 CLI 工具均为 Node.js 应用，用户若已安装任一工具则必然已安装 Node.js
- 线路切换功能（读写配置文件）不依赖 Node.js/npm，仅安装 CLI 工具功能依赖 npm
- 配置文件中的 base_url 字段名包含 `base_url`（不区分大小写），不使用其他命名变体
- 安装命令通过 npm 全局安装或官方安装脚本，在网络通畅时可成功
- 每次只有一个用户在操作本工具，不存在并发写入同一配置文件的场景
- 预设线路的 URL 是固定的，不随时间变化（如需更新则通过工具版本更新）
- 默认配置模板随工具分发（JSON 配置文件），安装后可直接写入而无需用户手动填写
- 安装包大小目标在 20MB 以内，不使用内嵌完整浏览器引擎的方案
