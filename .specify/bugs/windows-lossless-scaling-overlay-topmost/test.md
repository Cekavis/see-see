# Bug Verification: Lossless Scaling 下截图遮罩未置顶

- **Slug**: windows-lossless-scaling-overlay-topmost
- **Tested**: 2026-08-04
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

原生 topmost 重排路径、回归检查、完整测试、发布构建和 0.5.1 本地安装均通过；当前环境未运行 Lossless Scaling，原始实机复现尚未执行。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Lossless Scaling 全屏游戏中触发截图 | not-run | 当前环境没有对应运行场景 |
| New / updated tests | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle windows_capture_overlay_is_raised_after_show` | pass | 1 passed |
| Regression suite | `cargo test --manifest-path src-tauri/Cargo.toml`; `npm test` | pass | Rust 全套与前端 42 tests 通过 |
| Lint / type-check | `npm run build`; `npm run lint`; `npm run format:check`; `cargo fmt -- --check` | pass | 无错误 |
| Release build/install | `npm run tauri build`; NSIS `/S` | pass | 本地安装版本 0.5.1 |

## Output Excerpts

```text
test windows_capture_overlay_is_raised_after_show ... ok
Test Files  13 passed (13)
Tests       42 passed (42)
Finished 2 bundles
See See  0.5.1
```

## Residual Risks

- Lossless Scaling 的实际渲染层级和运行权限组合尚未在本机端到端验证。
- 若 Lossless Scaling 以更高完整性级别运行，Windows 可能限制较低权限进程覆盖其窗口；需要在用户实际启动方式下确认。

## Recommendation

保留修复并请用户在 Lossless Scaling 场景复测；遮罩能立即出现后即可将验证状态升级为 verified。
