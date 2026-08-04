# Bug Verification: 设置窗口开启时截图十字光标不可见

- **Slug**: windows-capture-cursor-hidden-settings-open
- **Tested**: 2026-08-04
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

应用内十字光标的显示、坐标更新和隐藏行为已由自动测试覆盖，完整测试、构建和 0.5.1 覆盖安装通过；Lossless Scaling + 设置窗口开启的实机复现仍待用户验证。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 设置窗口开启时在 Lossless Scaling 中截图 | not-run | 等待用户测试新安装的 corrective build |
| New / updated tests | `npm test -- --run src/views/CaptureOverlay.test.tsx` | pass | 2 tests passed |
| Regression suite | `npm test`; `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 前端 42 tests 与 Rust 全套通过 |
| Lint / type-check | `npm run build`; `npm run lint`; `npm run format:check`; `cargo fmt -- --check` | pass | 无错误 |
| Release build/install | `npm run tauri build`; NSIS `/S` | pass | corrective build 已覆盖安装为 0.5.1 |

## Output Excerpts

```text
Test Files  1 passed (1)
Tests       2 passed (2)
Test Files  13 passed (13)
Tests       42 passed (42)
Finished 2 bundles
```

## Residual Risks

- 实际 Lossless Scaling 画面上的光标视觉对比仍需用户确认。
- 20px 白色十字带黑色阴影覆盖常见明暗背景；若特定游戏画面仍不清晰，再调整尺寸或颜色。

## Recommendation

保持 partial，等待用户在“设置窗口开启”条件下复测；应用内十字可见后即可升级为 verified。
