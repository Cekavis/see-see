# Bug Assessment: Windows 截图遮罩向右下偏移

- **Slug**: windows-capture-overlay-offset
- **Created**: 2026-07-30
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 现在截图时overlay会比原画面向右偏移一点点，导致屏幕最左边留出一点后面的窗口，请你调查原因。（也会向下几个像素）

## Symptom

Windows 截图 overlay 的可见内容没有从显示器物理原点开始，而是整体向右、向下偏移；显示器最左侧和顶部因此露出后方窗口。预期 overlay 客户区与冻结的显示器截图逐像素覆盖。

## Reproduction

1. 在 Windows 启动应用。
2. 触发截图 overlay。
3. 对照屏幕左边缘和顶部，观察 overlay 内容向右、向下偏移，边缘露出后方窗口。

## Suspected Code Paths

- `src-tauri/src/commands.rs:736` — 创建无边框 overlay，但没有关闭 Tauri/TAO 默认启用的 undecorated window shadow。
- `src-tauri/src/commands.rs:752` — 将窗口外框左上角设置为显示器物理原点，并把客户区尺寸设置为显示器物理尺寸。
- `src-tauri/src/capture.rs:361` — xcap 从 Windows `DEVMODE.dmPosition` 与 `dmPelsWidth/Height` 读取显示器物理边界；这里的坐标与截图尺寸一致，不是偏移来源。
- `src/styles.css:1123` — overlay 使用 `position: fixed; inset: 0`，且全局 `body` margin 已清零，因此前端布局没有制造边缘空隙。

## Root Cause Hypothesis

**Confidence: high.** Tauri 2.11.5 使用的 TAO 0.35.3 在 Windows 上默认令 `decoration_shadow = true`。当前窗口同时设置了 `decorations(false)`，因此被 TAO 视为“带阴影的无边框窗口”。TAO 会在 `WM_NCCALCSIZE` 中给此类窗口的客户区加入隐藏 inset：左侧为 Windows frame thickness，顶部在 Windows 11 为按 DPI 缩放的约 1–2 像素。`set_position` 设置的是窗口外框位置，而 WebView 从 inset 后才开始；所以外框位于显示器原点时，实际 overlay 内容必然向右、向下偏移。现象方向和像素量均与该实现一致。

## Proposed Remediation

**Preferred**: 在截图 overlay 的 `WebviewWindowBuilder` 上显式调用 `.shadow(false)`。截图遮罩不需要阴影或圆角；关闭默认 undecorated shadow 后，客户区可直接从窗口物理原点开始，无需维护平台相关的手工负偏移。

**Alternatives**:

- 按运行时测得的 outer/client rect 差值反向移动窗口；这会依赖 Windows 版本与 DPI，复杂且仍保留无用阴影，不推荐。
- 使用 borderless fullscreen API；当前每个物理显示器各有独立 overlay，切换到 fullscreen 会扩大多显示器与焦点行为的变更面，不适合此缺陷。

**Files likely to change**:

- `src-tauri/src/commands.rs`
- `src-tauri/tests/desktop_lifecycle.rs`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- 添加聚焦回归检查，约束 capture window builder 同时使用 `decorations(false)` 与 `shadow(false)`。
- 运行 Rust 测试、前端构建和 Tauri Windows 构建。
- 在 Windows 100% 和至少一个高 DPI 缩放比例下手动确认 overlay 的左上边缘与显示器原点对齐。

## Risks & Considerations

- `.shadow(false)` 在 Linux 不受支持，但不会改变现有行为；此窗口创建路径不用于 macOS。
- 修复会去掉 Windows 11 对无边框窗口附加的白边和圆角，这正是全屏截图遮罩所需的行为。
- 自动化单元测试无法验证 DWM 的实际客户区边界，Windows 桌面端视觉检查仍是最终验证。

## Open Questions

- 无。
