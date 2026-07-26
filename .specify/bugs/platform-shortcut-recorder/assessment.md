# Bug Assessment: 平台默认快捷键与快捷键录入器失效

- **Slug**: platform-shortcut-recorder
- **Created**: 2026-07-25
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 现在默认的 Alt+Shift+A 在 macOS 上就不对，你应该对 Windows 和 macOS 分别写。其次，在macOS上我也无法设置新的快捷键，那个配置栏像是一个文本框，而不是符合直觉的点击后按新的快捷键就能自动记录并且生效。

## Symptom

macOS 与 Windows 共用写死的 `Alt+Shift+A` 默认值，未体现各平台的主修饰键习惯。设置页把快捷键暴露为自由文本输入框，用户必须猜测 Tauri 的快捷键语法并另行点击保存，不能通过直接按键完成录入和立即生效。

## Reproduction

1. 在 macOS 首次启动应用并打开桌面设置。
2. 观察截图快捷键为 `Alt+Shift+A`，而非 macOS 的 Command 组合键。
3. 点击“截图快捷键”配置栏并按下一个新组合键。
4. 配置栏按普通文本框处理输入，不会把组合键规范化、注册并立即保存。

## Suspected Code Paths

- `src-tauri/migrations/0001_init.sql:78` — 数据库种子值把所有平台的默认快捷键写死为 `Alt+Shift+A`。
- `src-tauri/src/database.rs:25` — 初始化与升级流程未按运行平台设置默认值，也未迁移旧默认值。
- `src/views/DesktopSettings.tsx:56` — 快捷键设置使用普通受控文本输入框，仅监听 `onChange`。
- `src/views/DesktopSettings.tsx:63` — 必须点击独立保存按钮后才调用快捷键注册 IPC。

## Root Cause Hypothesis

**高置信度**。持久化层只有一个跨平台硬编码默认值，界面层也没有快捷键捕获模型：它接受任意字符串，不监听 `keydown` 的修饰键与主键，因此 macOS 用户既看不到符合平台习惯的默认组合，也无法按直觉录入可被 Tauri 解析的快捷键。

## Proposed Remediation

**Preferred**: 在 Rust 持久化初始化中明确维护 Windows（`Ctrl+Shift+X`）与 macOS（`Command+Shift+X`）默认值，并在数据库版本升级时只把历史遗留的 `Alt+Shift+A` 默认值迁移为当前平台默认值。把前端文本框替换为只读、可聚焦的快捷键录入控件；获得焦点后捕获下一次有效 `keydown`，规范化为 Tauri 快捷键字符串并立即调用现有 `setCaptureShortcut`。注册失败时恢复已生效的旧值并继续显示错误。

快捷键录入应忽略单独按下的修饰键，支持字母、数字、功能键和常见命名键，阻止浏览器/系统页面对该组合键的默认处理，并在保存期间禁用重复录入。

**Files likely to change**:

- `src-tauri/src/database.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src/views/DesktopSettings.tsx`
- `src/views/Settings.desktop.test.tsx`
- `src/styles.css`

**Tests to add or update**:

- 验证 Windows 与 macOS 默认快捷键映射为各自明确的字符串。
- 验证录入控件捕获 macOS Command 组合键后自动调用保存 API。
- 验证冲突时回退到旧快捷键，单独修饰键不会触发保存。

## Risks & Considerations

- 迁移必须只替换历史默认值，不能覆盖用户自定义快捷键。
- 浏览器 `KeyboardEvent` 的命名必须转换为 Tauri/global-hotkey 可解析的格式。
- macOS 全局快捷键注册及系统级冲突仍需在真实应用中人工验证。

## Open Questions

- 无。
