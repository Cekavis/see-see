# Bug Assessment: macOS 旧进程的结果窗落到 Desktop Space

- **Slug**: macos-result-existing-process-fullscreen-space
- **Created**: 2026-08-14
- **Source**: pasted text + 多轮真机复现
- **Verdict**: valid
- **Severity**: high

## Report

See See 驻留在后台（Desktop Space）时，用户进入另一个应用独占的原生全屏 Space 并触发截图翻译，结果窗口不会出现在当前全屏 Space，而是在 Desktop Space 静默创建。用户在 ChatGPT、Zen 等全屏应用中均能复现。

## Root Cause

结果窗口原本使用 `NSWindowCollectionBehaviorMoveToActiveSpace | FullScreenAuxiliary`。`MoveToActiveSpace` 会把新窗口拖到 See See 自己所在的 Space（Desktop Space）。

更关键的是：See See 以普通（`NSApplicationActivationPolicyRegular`）身份运行时，即使给窗口加 `CanJoinAllSpaces`，窗口也无法加入“其他应用独占的原生全屏 Space”。这是 macOS 对普通应用的约束（参考 Tauri #11488、Wails #4756）：要让窗口进入别人的全屏 Space，应用需要以 accessory（菜单栏工具）身份运行。

## Proposed Remediation

1. 应用启动时把激活策略设为 `Accessory`（对应 `NSApplicationActivationPolicyAccessory`）。
2. 结果窗口策略改为 `CanJoinAllSpaces | FullScreenAuxiliary`。

这样结果窗口会加入所有 Space（含当前全屏 Space），且不会因为激活 See See 而被 AppKit 拖回 Desktop Space。

## Files Likely to Change

- `src-tauri/src/lib.rs`
- `src-tauri/src/windowing.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- 版本同步文件（`package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json`）
