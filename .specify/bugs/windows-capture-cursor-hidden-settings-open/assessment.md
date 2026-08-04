# Bug Assessment: 设置窗口开启时截图十字光标不可见

- **Slug**: windows-capture-cursor-hidden-settings-open
- **Created**: 2026-08-04
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 之前的描述可能有误：框选界面实际已经显示。问题是十字鼠标不可见，但仍能拖动框选。See See 设置窗口开着时会复现；设置窗口没开着时已经正常。

## Symptom

Windows + Lossless Scaling 场景中，截图遮罩可见且持续收到 pointer 事件，但当 See See 主设置窗口同时开启时，系统十字光标不可见。预期无论其他窗口状态如何，用户都能看见当前框选位置。

## Reproduction

1. 打开 See See 设置窗口。
2. 启用 Lossless Scaling 并触发截图。
3. 移动鼠标：十字光标不可见，但拖动后选区边框正常出现。
4. 关闭或隐藏 See See 设置窗口后再次截图，十字光标正常。

## Suspected Code Paths

- `src/styles.css:.capture-overlay` — 当前仅通过 `cursor: crosshair` 请求 Windows/WebView2 原生光标，没有应用内可见光标。
- `src/views/CaptureOverlay.tsx:handleMove()` — pointer 坐标始终可用，但只在拖动时更新选区，没有把坐标用于绘制光标。
- `src/views/CaptureOverlay.test.tsx` — 已验证 pointer capture 和选区计算，未验证光标可见性与位置。

## Root Cause Hypothesis

**Confidence: high.** 用户确认遮罩可见、pointer 事件正常且能完成框选，因此不是窗口层级或输入路由问题。当前十字光标完全依赖 Windows 原生 cursor；原生 cursor 可被 Lossless Scaling 等前台程序通过系统显示状态隐藏，CSS 只能选择光标形状，不能保证其显示。设置窗口状态改变了前台/光标恢复时序，暴露了该依赖。

## Proposed Remediation

**Preferred**: 将 capture overlay 的原生 cursor 设为 `none`，在 overlay 内添加一个不接收 pointer 事件的 DOM 十字光标。复用已有 pointer move 事件直接更新该元素位置，不调用 Win32 `ShowCursor`，不修改系统光标显示计数，也不隐藏或恢复设置窗口。

**Files likely to change**:

- `src/views/CaptureOverlay.tsx`
- `src/views/CaptureOverlay.test.tsx`
- `src/styles.css`

**Tests to add or update**:

- 更新 CaptureOverlay 测试，约束首次移动后应用内十字光标显示并跟随 `clientX/clientY`。
- 运行前端测试、构建、lint、格式检查、Rust 全套、Tauri 0.5.1 发布构建和覆盖安装。

## Risks & Considerations

- 应隐藏原生 cursor，避免正常环境中出现双十字。
- DOM 光标必须 `pointer-events: none`，不能影响现有框选事件。
- 多显示器切换时离开一个 overlay 应隐藏其光标，避免旧显示器残留十字。
- 本次仍属于同一 0.5.1 session objective 的 corrective fix，不再次递增版本。

## Open Questions

- 无。
