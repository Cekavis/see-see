# Bug Assessment: macOS 开机启动不出现在登录项

- **Slug**: login-items-autostart
- **Created**: 2026-07-26
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

> 当前的开机启动打开后似乎不会添加到login items里，请你调查

## Symptom

在 macOS 中打开“开机启动”后，See See 不会出现在系统设置“通用 > 登录项与扩展 > 登录时打开”列表。当前实现可能仍通过 LaunchAgent 在下次登录时启动应用，因此“没有登录项”已确认是实现选择导致的，而“完全不会开机启动”尚未复现。

## Reproduction

1. 安装并启动 `/Applications/See See.app` 0.3.2。
2. 在 See See 设置中打开“开机启动”。
3. 打开 macOS 系统设置“通用 > 登录项与扩展”。
4. 观察“登录时打开”列表中没有 See See。
5. [NEEDS CLARIFICATION: 注销并重新登录后，See See 是否仍会由 LaunchAgent 启动。]

调查时本机数据库中的 `autostart` 为 `0`，`~/Library/LaunchAgents`、System Events 登录项和 `sfltool dumpbtm` 均没有 See See 记录。这只能说明调查时功能处于关闭状态，不能证明此前的开启动作失败。

## Suspected Code Paths

- `src-tauri/src/lib.rs:54` — 使用 `tauri_plugin_autostart::Builder::new().build()` 的默认配置；锁定的插件 2.5.1 在 macOS 上默认选择 `MacosLauncher::LaunchAgent`，不是系统登录项。
- `src-tauri/src/commands.rs:517` — 开关直接调用插件的 `enable`/`disable`，没有 macOS 专用的登录项注册路径。
- `src-tauri/src/lib.rs:76` — 启动时的状态同步只检查所选后端；当前后端的 `is_enabled` 实际只检查 LaunchAgent plist 是否存在。
- `src-tauri/tests/desktop_lifecycle.rs:35` — 只验证系统操作成功后才写数据库，没有覆盖真实 macOS 注册方式或系统可见性。
- `specs/001-screenshot-ai-translation/validation/us5.md:9` — 已明确记录 macOS 开机启动的人工验证仍未完成。

## Root Cause Hypothesis

**高置信度**：插件初始化未指定 macOS launcher，因此使用默认 `LaunchAgent`。锁定的 `tauri-plugin-autostart` 2.5.1 通过 `auto-launch` 0.5.0 在 `~/Library/LaunchAgents/<app_name>.plist` 写入直接执行 `See See.app/Contents/MacOS/see-see` 的配置；这条路径不会把应用加入“登录时打开”列表。由此可以确定用户观察到的“没有添加到 Login Items”属实。LaunchAgent 理论上会在后续登录时由 `launchd` 加载，所以目前证据不足以断言实际自动启动也失效。

## Proposed Remediation

**Preferred**: 鉴于应用最低支持 macOS 14，在 macOS 上改用 Apple 的 `SMAppService.mainApp` 注册/注销主应用登录项，并以 `SMAppService.status` 作为启动时状态同步来源。这样使用系统当前推荐的登录项 API，See See 会作为应用出现在“登录时打开”，系统拒绝或要求用户批准时也能返回明确状态。Windows 继续使用现有 Tauri autostart 后端。

**Alternatives**:

- 显式配置插件的 `MacosLauncher::AppleScript`。改动最小，插件会通过 System Events 创建可见登录项，但依赖 Apple Events/自动化授权和较旧的脚本接口；本机从终端读取 System Events 登录项返回 `-10827`，应在采用前验证打包应用上下文。
- 保留 LaunchAgent，并在 UI 中明确说明它属于后台启动项而非“登录时打开”。这可以修正预期，但不能满足用户希望在 Login Items 中看到应用的目标，也仍需验证新 macOS 版本上的加载行为。

**Files likely to change**:

- `src-tauri/src/lib.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/settings.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/tests/desktop_lifecycle.rs`
- `specs/001-screenshot-ai-translation/validation/us5.md`

**Tests to add or update**:

- 抽象系统自启动注册器，并单测 enable、disable、拒绝/失败、状态同步及数据库回滚。
- 在已安装的签名应用上验证打开后出现在“登录时打开”，关闭后消失。
- 注销并重新登录，验证仅在开关打开时自动启动；同时验证移动或重装应用后的状态。
- 在 Windows 上回归现有启动项注册，确保平台分支没有改变原行为。

## Risks & Considerations

- `SMAppService` 的状态包含 enabled、requiresApproval、notRegistered、notFound 等多种情况，不能继续简单映射为单一布尔值而隐藏“需用户批准”。
- 注册主应用要求从正确安装位置和打包上下文运行；开发模式不适合作为最终验证依据。
- 从旧版本升级时应清理可能遗留的 `~/Library/LaunchAgents/See See.plist`，避免新旧机制重复启动。
- 当前通用错误“无法更新开机启动状态”丢失了平台错误细节，修复时应增加脱敏日志或可操作的用户提示。
- 本次调查使用 macOS 27.0 预览系统；修复后仍需按最低支持版本 macOS 14 做一次实机验证。

## Open Questions

- [NEEDS CLARIFICATION: 用户当时打开开关后，`~/Library/LaunchAgents/See See.plist` 是否曾成功生成。]
- [NEEDS CLARIFICATION: 用户关注的是系统列表可见性，还是注销后 See See 也没有自动启动。]
- [NEEDS CLARIFICATION: 发行流程是否使用代码签名；`SMAppService` 应在与正式分发一致的签名包上验证。]
