# Bug Assessment: macOS 快捷键录入无响应与冗余说明文案

- **Slug**: shortcut-recorder-focus-copy
- **Created**: 2026-07-25
- **Source**: pasted text and direct user reproduction
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 我试了，现在点击之后显示 请按新的快捷键，但按什么都没反应，请你修复。另外你应该删掉“登录系统后自动启动 See See。”这种重复无意义的副标题，你做一个全面检查，并且在 AGENTS.md 里记录这项原则。

## Symptom

macOS 中快捷键按钮可以进入“请按新的快捷键…”状态，但随后按键不会被捕获或保存。界面还存在至少一处说明文案仅复述控件标题，没有增加约束、后果或操作信息，造成视觉和认知噪音。

## Reproduction

1. 在 macOS 设置的“常规”页点击“截图快捷键”。
2. 观察按钮显示“请按新的快捷键…”。
3. 按任意新的组合键。
4. 按钮保持录制状态，快捷键没有更新或生效。
5. 观察“开机启动”下方同时显示“登录系统后自动启动 See See。”，其含义与标题重复。

## Suspected Code Paths

- `src/views/DesktopSettings.tsx:159` — `keydown` 只绑定在按钮本身，依赖点击后按钮获得并保持键盘焦点；macOS WebView 的按钮点击焦点行为并不可靠。
- `src/views/DesktopSettings.tsx:210` — “登录系统后自动启动 See See。”只复述“开机启动”。
- `src/views/Settings.desktop.test.tsx` — 当前测试直接把 `keydown` 派发给按钮，无法暴露真实 WebView 中事件目标落在窗口/文档的问题。
- `src/views/*.tsx` 与 `src/components/*.tsx` — 需要按统一标准复查帮助文案、空状态说明、状态说明与确认文案。
- `AGENTS.md` — 尚未记录“辅助文案必须提供新增信息”的界面写作原则。

## Root Cause Hypothesis

**高置信度**。录制逻辑将 `onKeyDown` 放在 `<button>` 上；macOS WebKit 点击按钮时不保证像文本框那样赋予键盘焦点，因此录制状态虽已更新，但后续事件不会到达该按钮。测试也复刻了错误假设，直接以按钮为事件目标。冗余文案则源于缺少明确的界面写作门槛，导致标题被同义句再次解释。

## Proposed Remediation

**Preferred**: 录制开始后，在 `window` 的捕获阶段临时监听 `keydown`，从任何事件目标接收组合键；保存成功、失败或按 Esc 取消后立即移除监听。键名转换同时以 `KeyboardEvent.code` 为主、`key` 为回退，兼容 macOS WebView/输入法可能提供空 `code` 的情况。测试必须把按键派发到 `window`，证明实现不依赖按钮焦点。

全面审查当前 React 界面的辅助文案：删除仅复述控件标题或按钮含义的句子；保留并精简提供动态状态、安全/隐私影响、费用、格式约束、不可逆后果、回退方式或非显然操作的信息。在 `AGENTS.md` 的编码风格中固化这一标准。

**Files likely to change**:

- `src/views/DesktopSettings.tsx`
- `src/views/Settings.desktop.test.tsx`
- `src/views/DesktopSettings.copy.test.tsx`（如需要独立文案回归测试）
- `AGENTS.md`
- `.specify/bugs/shortcut-recorder-focus-copy/fix.md`

**Tests to add or update**:

- 从 `window` 派发 Command 组合键后自动保存并退出录制状态。
- `KeyboardEvent.code` 不可用时用 `key` 生成可解析组合。
- 单独修饰键仍不保存，Esc 取消并移除全局监听。
- “开机启动”不再渲染同义副标题，其他保留的帮助文案均提供独立信息。

## Risks & Considerations

- 窗口级监听必须仅在录制期间存在，并在组件卸载时可靠清理，避免吞掉其他页面快捷键。
- 捕获阶段应阻止已接受组合的默认行为；单独修饰键也不能意外退出录制。
- 系统保留的 macOS 组合键可能在到达 WebView 前被操作系统截获，不能承诺可录入所有系统快捷键。
- 文案审查不能误删权限状态、隐私、费用或不可逆操作等决策所需信息。

## Open Questions

- 无。
