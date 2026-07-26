# Bug Assessment: macOS 全屏 Space 截图窗口跳转

- **Slug**: macos-fullscreen-space-capture
- **Created**: 2026-07-26
- **Source**: pasted text
- **Verdict**: valid
- **Severity**: medium

## Report (verbatim or summarized)

> 现在功能确实可用了，但我预期的是，按下快捷键后，可以直接在窗口上进行截图，包括全屏的窗口，不用切到其他工作区再框选，框选完毕后在同一个工作区弹出一个紧凑的小窗展示结果。

## Symptom

在普通窗口或 macOS 全屏窗口所在的当前 Space 触发截图后，See See 的遮罩或结果窗口可能出现在应用自己的另一个 Space，迫使用户切换工作区才能框选或查看结果。预期是冻结并覆盖当前可见内容（包括全屏窗口），完成框选后仍在同一 Space 显示紧凑结果窗。

## Reproduction

1. 在 macOS 26+ 将另一个应用窗口切换为系统全屏，使其进入独立 Space。
2. 保持该全屏窗口可见，按 See See 全局截图快捷键。
3. 当前实现创建普通置顶遮罩窗口，但没有配置可加入全屏 Space 的 `NSWindowCollectionBehavior`，遮罩可能出现在 See See 所属 Space。
4. 完成框选后，结果窗口以默认 `620×720` 创建，也没有移动到当前活动 Space，可能再次触发 Space 切换或留在其他工作区。

## Suspected Code Paths

- `src-tauri/src/commands.rs:create_capture_windows()` — 遮罩只设置 `always_on_top`，没有 `CanJoinAllSpaces`、`FullScreenAuxiliary` 或适合全屏覆盖的窗口级别。
- `src-tauri/src/commands.rs:create_result_window()` — 结果窗口创建时立即可见，默认尺寸为 `620×720`，没有在显示前应用 `MoveToActiveSpace` 与 `FullScreenAuxiliary`。
- `src-tauri/src/capture/macos.rs:capture_all()` — 在遮罩创建前通过 ScreenCaptureKit 冻结显示器图像，说明全屏内容读取链路不是本问题的主要故障点。
- `src-tauri/src/commands.rs:finish_capture()` — 关闭遮罩后创建结果窗；没有保存或传递窗口 Space 信息，因此结果窗完全依赖 AppKit 默认放置策略。

## Root Cause Hypothesis

**高置信度**。Tauri 的 `always_on_top(true)` 只处理窗口层级，不会自动把窗口标记为可进入其他应用的全屏 Space。当前遮罩和结果窗都缺少 macOS collection behavior；结果窗还在应用策略配置前以默认可见状态创建，因此 AppKit 会按 See See 进程的常规窗口规则选择 Space。ScreenCaptureKit 已经在创建遮罩之前捕获当前显示器，所以无需让用户先退出全屏或切换工作区，缺失的是原生窗口呈现策略。

## Proposed Remediation

**Preferred**: 增加隔离的 macOS 窗口策略适配器。遮罩在首次显示前设置 `CanJoinAllSpaces | FullScreenAuxiliary | Stationary | IgnoresCycle`，并提升到适合全屏截图覆盖的原生窗口级别；非 macOS 保持当前 Tauri 行为。结果窗先以隐藏状态创建，再设置 `MoveToActiveSpace | FullScreenAuxiliary`，居中后显示并聚焦，使它留在完成框选时的活动 Space，而不是切回 See See 原来的工作区。

将结果窗默认尺寸从 `620×720` 收紧到紧凑尺寸，同时保留现有 `420×360` 最小尺寸、滚动内容、流式状态、复制、取消和置顶控制。截图数据、权限、坐标与图像合成链路保持不变。

**Alternatives**:

- 仅使用 Tauri `visible_on_all_workspaces(true)`：能添加 `CanJoinAllSpaces`，但不会添加 `FullScreenAuxiliary`，不足以可靠进入其他应用的系统全屏 Space。
- 退出或最小化当前全屏应用后截图：违背用户期望，且会改变被截图内容，不应采用。

**Files likely to change**:

- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/windowing.rs`（新增平台适配器）
- `src-tauri/tests/desktop_lifecycle.rs`
- `README.md` 或 `specs/001-screenshot-ai-translation/quickstart.md`
- 同步版本文件（当前会话目标已经是 `0.3.3`，不重复递增）

**Tests to add or update**:

- 窗口策略单元测试：遮罩包含 join-all-spaces/full-screen-auxiliary/stationary/ignore-cycle 标志，结果窗包含 move-to-active-space/full-screen-auxiliary，且互斥 Space 标志不会混用。
- 结果窗口契约测试：默认尺寸紧凑，最小尺寸继续满足现有 `420×360` 可访问性约束。
- 原生 macOS 回归：从普通窗口、系统全屏窗口分别触发快捷键，遮罩出现在当前 Space；框选后结果窗在同一 Space 显示。
- 多显示器回归：每个显示器遮罩仍使用既有物理坐标和尺寸，并且 Esc/完成会关闭全部遮罩。

## Risks & Considerations

- `CanJoinAllSpaces` 与 `MoveToActiveSpace` 不能同时用于同一个窗口，适配器必须按窗口角色生成互斥策略。
- 原生 `NSWindow` 必须在 AppKit 主线程、窗口首次显示前配置，避免短暂 Space 跳转或闪烁。
- 过高窗口级别可能覆盖菜单栏或系统安全界面；只对短生命周期截图遮罩使用，结果窗继续遵守用户的置顶设置。
- 结果窗缩小后必须保留长文本滚动和通知可访问性，不能裁掉取消、复制或置顶控制。
- Windows 不应继承 macOS Space 行为；跨平台窗口创建和现有 xcap 路径需保持不变。

## Open Questions

- 无。用户已明确要求当前全屏 Space 内完成截图，并在同一 Space 显示紧凑结果窗。
