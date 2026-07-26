# Bug Verification: macOS 双显示器遮罩固定原生 frame

- **Slug**: macos-multidisplay-capture-space-switch-2
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: failed

## Summary

两个遮罩的原生 frame 已分别覆盖各自显示器，Esc 也会同时关闭它们；但主屏当前为 Codex 的全屏 Space，视觉截图中该屏没有遮罩和选区提示。原始“无需切换工作区即可框选全屏窗口”的症状仍可复现。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 安装 0.3.5，发送 `Command+Shift+2`，读取窗口 frame 并分别截取两块显示器 | fail | frame 正确；第二屏遮罩可见，主屏全屏 Codex Space 没有遮罩 |
| Native frame contract | `CGWindowListCopyWindowInfo` | pass | `1920×1080 @ (0,0)` 与 `1710×1107 @ (99,1080)` |
| Cancel behavior | 用户按 Esc 后再次查询原生窗口 | pass | `capture_windows=0` |
| Updated Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` | pass | 7 passed |
| Lint | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | pass | 无警告 |

## Output Excerpts

```text
owner=See See name=See See Capture layer=101 bounds=["Y": 1080, "Height": 1107, "X": 99, "Width": 1710]
owner=See See name=See See Capture layer=101 bounds=["Y": 0, "Height": 1080, "X": 0, "Width": 1920]
capture_windows=0
```

显示器截图：

- 主屏 `3840×2160`：全屏 Codex 正常显示，没有暗色遮罩和“拖动选择区域”提示。
- 第二屏 `3420×2214`：暗色遮罩与“拖动选择区域 · Esc 取消”提示正常显示。

## Residual Risks

- `CanJoinAllSpaces` 不等于能加入另一应用的全屏集合；仅检查窗口 frame 会产生错误通过。
- 后续验证必须同时检查原生 frame 和每块屏幕的实际视觉状态。

## Recommendation

重新打开此 bug。按新的 macOS 26 SDK 证据重新评估，在遮罩策略中加入允许跨应用全屏 Space 的 `CanJoinAllApplications`，重新构建安装后再次执行双屏视觉复验。
