# Bug Assessment: Windows 截图遮罩显示动画

- **Slug**: windows-capture-overlay-animation
- **Created**: 2026-07-30
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: low

## Report (verbatim or summarized)

> 现在正常了，但是请你一并修复一个问题：截图遮罩跳出来有一个可能是系统动画，我希望改成直接出现

## Symptom

Windows 截图遮罩显示时带有短暂的系统窗口过渡效果。预期遮罩在完成定位后直接出现，不播放窗口显示动画。

## Reproduction

1. 在 Windows 安装版触发截图。
2. 观察遮罩从隐藏状态显示到屏幕上的过程。
3. 遮罩带有短暂的系统过渡，而不是立即覆盖屏幕。

## Suspected Code Paths

- `src-tauri/src/commands.rs:743` — capture window 先以 `visible(false)` 创建，完成定位和尺寸设置后再显示。
- `src-tauri/src/windowing.rs:115` — Windows 路径直接调用 Tauri `window.show()`，没有在显示前关闭该窗口的 DWM transition。
- `src/styles.css:1123` — capture overlay 本身没有 CSS transition 或 animation，因此前端样式不是该系统级显示过渡的来源。

## Root Cause Hypothesis

**Confidence: high.** Tauri/TAO 的 Windows `show()` 路径最终调用 `ShowWindow(..., SW_SHOW)`；当前 capture window 没有设置 `DWMWA_TRANSITIONS_FORCEDISABLED`，所以 Windows DWM 可以对隐藏到显示的顶层窗口应用系统过渡。由于遮罩必须先隐藏创建以避免默认尺寸和位置闪现，不能简单改为初始可见；应在现有 `show()` 前按窗口禁用 DWM transition。

## Proposed Remediation

**Preferred**: 在 Windows 的 `show_capture_window` 路径中，取得 capture window HWND，调用 `DwmSetWindowAttribute` 将 `DWMWA_TRANSITIONS_FORCEDISABLED` 设为 true，然后保持现有 `window.show()`。使用 Tauri 已采用的 `windows` crate 版本与原生 API，不增加延时或前端动画逻辑。

**Alternatives**:

- 创建时直接设为可见；窗口构建后还需要设置物理位置和尺寸，会重新引入错误位置/尺寸闪现。
- 显示前固定等待；无法关闭系统过渡，并会增加可感知延迟，不推荐。

**Files likely to change**:

- `src-tauri/src/windowing.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`

**Tests to add or update**:

- 添加聚焦回归检查，约束 Windows capture show 路径在 `window.show()` 前使用 `DWMWA_TRANSITIONS_FORCEDISABLED`。
- 运行 Rust 测试、前端检查和 Tauri Windows 发布构建。
- 在安装版触发截图，确认遮罩直接出现，并复查无边框对齐仍正常。

## Risks & Considerations

- DWM 属性只应用于 capture window，不应改变主窗口或结果窗口动画。
- 原生 API 调用应在 Windows 条件编译下，保持 macOS 和 Linux 构建不受影响。
- 若 DWM 调用失败，应沿用现有 capture window 错误路径，不静默回退到带动画显示。

## Open Questions

- 无。
