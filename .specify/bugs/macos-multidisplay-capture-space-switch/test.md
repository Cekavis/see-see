# Bug Verification: macOS 双显示器截图遮罩原位显示

- **Slug**: macos-multidisplay-capture-space-switch
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: failed

## Summary

逻辑尺寸修复在自动化合同中通过，但安装后的 0.3.5 原生验证发现 `MoveToActiveSpace` 会把两个遮罩都搬到同一个活动显示器，因此原始双屏错位症状尚未完全解决。该实现不应提交，需保留逻辑几何修复并重新评估 Space 策略和首次显示时序。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 安装 0.3.5，发送 `Command+Shift+2`，读取 See See on-screen window frames | fail | 两个遮罩尺寸正确，但 X 均为 `99`，被移到同一活动显示器 |
| New Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` | pass | 7 passed；证明逻辑几何合同，但未能模拟 AppKit 的实际 Space 搬移 |
| New frontend test | `npm test -- src/views/CaptureOverlay.test.tsx` | pass | 2 passed；1:1 逻辑选区换算正确 |
| Regression suites | `npm test`; `cargo test --manifest-path src-tauri/Cargo.toml`; `npm run test:e2e` | pass | 前端 38 项、Rust 全量和浏览器主流程通过 |
| Lint / build | `npm run lint`; `npm run build`; `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | pass | 静态检查和构建通过 |

## Output Excerpts

```text
name=See See Capture bounds=["Width": 1710, "X": 99, "Height": 1107, "Y": 823] layer=101
name=See See Capture bounds=["Width": 1920, "X": 99, "Height": 1080, "Y": 1407] layer=101
```

预期两个遮罩分别固定在 CoreGraphics displays `0,0 1920×1080` 与 `99,1080 1710×1107`，而不是都以 `X=99` 出现在第二显示器。

## Residual Risks

- `MoveToActiveSpace` 对多显示器窗口按应用当前活动显示器搬移，不能作为每显示器遮罩策略。
- Tauri 的 `set_position` / `set_size` 和自定义主线程显示分别排队，首次显示前的原生 frame 仍存在时序不确定性。

## Recommendation

不要交付当前实现。保留 macOS logical geometry 与 selection scale `1.0`，重新运行 `/speckit-bug-assess`，将遮罩恢复为固定 frame 的 `CanJoinAllSpaces | FullScreenAuxiliary`，并在同一个 AppKit 主线程回调中设置原生 frame、策略和显示顺序。
