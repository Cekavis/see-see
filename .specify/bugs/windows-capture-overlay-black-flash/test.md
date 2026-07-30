# Bug Verification: Windows 截图遮罩黑屏闪烁

- **Slug**: windows-capture-overlay-black-flash
- **Tested**: 2026-07-30
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

自动化检查证明窗口显示已从创建路径延后到 PNG 解码和背景提交之后，完整回归、发布构建和最终 0.4.3 安装均通过。最终安装版尚未由用户再次目视确认，因此按流程标记为 partial。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 最终 Windows 0.4.3 安装版触发截图 | not-run | 等待用户目视确认首次可见帧 |
| New frontend test | `npx vitest run src/views/CaptureOverlay.test.tsx` | pass | 2 passed；decode 前不显示 |
| New Rust test | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle capture_overlay_waits_for_frontend_frame_readiness` | pass | 1 passed |
| Regression suite | `npm test`; `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 前端 42 tests；Rust 全套通过 |
| Lint / type-check | `npm run build`; `npm run lint`; `npm run format:check`; `cargo fmt -- --check` | pass | 无错误 |
| Release build/install | `npm run tauri build`; NSIS `/S` | pass | 最终安装版本 0.4.3 |

## Output Excerpts

```text
Test Files  13 passed (13)
Tests       42 passed (42)
test capture_overlay_waits_for_frontend_frame_readiness ... ok
Finished 2 bundles
ProductVersion 0.4.3
```

## Residual Risks

- WebView2 的最终可见合成帧仍需一次人工目视确认；代码和测试已移除已知的提前显示路径。
- 图片读取或解码失败时会显示窗口以呈现错误通知，错误路径仍可能使用黑色背景，这是刻意的恢复行为。

## Recommendation

Hold only for one installed-app visual check. If the overlay appears directly with the frozen screenshot, promote this result to verified and close the bug.
