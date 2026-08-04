# Bug Verification: Lossless Scaling 下截图遮罩未获取前台焦点

- **Slug**: windows-lossless-scaling-overlay-focus
- **Tested**: 2026-08-04
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

前台聚焦调用、回归检查、完整测试、发布构建和 0.5.1 覆盖安装均通过；新的 corrective build 尚未由用户在 Lossless Scaling 场景复测。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Lossless Scaling 全屏游戏中触发截图 | not-run | 等待用户测试新的 corrective build |
| New / updated tests | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle windows_capture_overlay_is_raised_and_focused_after_show` | pass | 1 passed |
| Regression suite | `cargo test --manifest-path src-tauri/Cargo.toml`; `npm test` | pass | Rust 全套与前端 42 tests 通过 |
| Lint / type-check | `npm run build`; `npm run lint`; `npm run format:check`; `cargo fmt -- --check` | pass | 无错误 |
| Release build/install | `npm run tauri build`; NSIS `/S` | pass | corrective build 已覆盖安装为 0.5.1 |

## Output Excerpts

```text
test windows_capture_overlay_is_raised_and_focused_after_show ... ok
Test Files  13 passed (13)
Tests       42 passed (42)
Finished 2 bundles
See See  0.5.1
```

## Residual Risks

- 新增前台聚焦仍需用户在 Lossless Scaling 实际渲染层上验证。
- 若 Lossless Scaling 以更高完整性级别运行，Windows 可能拒绝较低权限进程取得前台。

## Recommendation

保持 partial，等待用户用新的 0.5.1 corrective build 复测；若仍失败，再采集前台窗口句柄、进程完整性级别和 Win32 返回值后重新评估。
