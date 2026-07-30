# Bug Verification: Windows 截图遮罩向右下偏移

- **Slug**: windows-capture-overlay-offset
- **Tested**: 2026-07-30
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

0.4.3 Windows 安装版不再出现 capture overlay 向右下偏移或屏幕边缘露底。用户完成真实截图复测并确认恢复正常，自动化回归与发布构建也通过。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Windows 安装版触发截图并观察边缘 | pass | 用户确认“现在正常了” |
| New / updated test | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle capture_overlay_disables_undecorated_window_shadow` | pass | 1 passed |
| Regression suite | `npm test`; `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 前端 41 tests；Rust 全套通过 |
| Lint / type-check | `npm run build`; `npm run lint`; `npm run format:check`; `cargo fmt -- --check` | pass | 无错误 |
| Release build | `npm run tauri build` | pass | 生成 0.4.3 MSI 与 NSIS |
| Local installation | NSIS 静默安装并读取版本 | pass | Display/File/Product version 均为 0.4.3 |

## Output Excerpts

```text
test capture_overlay_disables_undecorated_window_shadow ... ok
Test Files  13 passed (13)
Tests       41 passed (41)
Finished 2 bundles
DisplayVersion 0.4.3
```

## Residual Risks

- 本轮用户确认覆盖当前 Windows 显示器配置；多显示器坐标与选区合成继续由现有 Rust 测试覆盖。

## Recommendation

Close the bug — verified end-to-end on Windows.
