# Bug Fix: macOS 当前 Space 全屏截图

- **Slug**: macos-fullscreen-space-capture
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

截图遮罩现在会以 macOS 全屏辅助窗口加入当前 Space，结果窗则在隐藏状态下完成原生策略配置后移动到活动 Space 再显示。结果窗默认尺寸同时收紧为 `460×500`，保留原有 `420×360` 最小尺寸和滚动布局。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/windowing.rs` | added | 隔离窗口角色、Space 策略、原生 AppKit 配置和紧凑尺寸契约 |
| `src-tauri/src/commands.rs` | modified | 遮罩及结果窗在显示前应用角色对应策略 |
| `src-tauri/src/lib.rs` | modified | 导出窗口策略模块 |
| `src-tauri/Cargo.toml` | modified | macOS 直接声明最小 `objc2-app-kit` `NSWindow` 依赖 |
| `src-tauri/Cargo.lock` | modified | 记录直接 AppKit 依赖及 `0.3.4` 包版本 |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` | modified | 将本次新修复的同步版本提升到 `0.3.4` |
| `src-tauri/tests/desktop_lifecycle.rs` | added tests | 固定互斥 Space 行为、全屏辅助标志和尺寸约束 |
| `README.md` | modified | 记录 macOS 当前普通或全屏 Space 内完成截图的能力 |
| `specs/001-screenshot-ai-translation/quickstart.md` | modified | 增加普通与系统全屏 Space 的人工验收步骤 |

## Diff Highlights

- 遮罩：`CanJoinAllSpaces | FullScreenAuxiliary | Stationary | IgnoresCycle`，并仅为短生命周期遮罩使用弹出菜单级原生层级。
- 结果窗：`MoveToActiveSpace | FullScreenAuxiliary`，隐藏创建后在 AppKit 主线程居中、显示并聚焦。
- Windows：继续使用现有 Tauri 显示路径，不应用 macOS 原生策略。

## Tests Added or Updated

- `src-tauri/tests/desktop_lifecycle.rs::macos_capture_and_result_windows_use_distinct_space_policies` — 固定遮罩与结果窗的互斥 Space 策略及全屏辅助行为。
- `src-tauri/tests/desktop_lifecycle.rs::result_window_defaults_are_compact_and_keep_accessible_minimums` — 固定紧凑默认尺寸和现有最小尺寸。

## Local Verification

- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` → 6 passed。
- Commands run: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → passed。
- Manual checks: 留待 `/speckit-bug-test` 使用安装后的 macOS 应用验证普通窗口、系统全屏 Space 和多显示器路径。

## Deviations from Assessment

- 初始评估误把上一任务已经发布的 `0.3.3` 当作同一会话目标，因而建议不重复递增。用户指出这是新的改进后，已按仓库规则将本次修复同步提升为 `0.3.4`。

## Follow-ups

- 在 macOS 26+ 安装包中分别从普通窗口和系统全屏窗口触发快捷键，确认遮罩及紧凑结果窗都留在当前 Space。
- 在多显示器布局中复验遮罩位置、Esc 取消及完成后全部关闭。
