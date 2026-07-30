# Bug Verification: Windows 截图遮罩显示动画

- **Slug**: windows-capture-overlay-animation
- **Tested**: 2026-07-30
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

Windows capture overlay 的系统窗口动画不再出现。用户在安装版实机确认“动画没有了”，DWM 回归检查、完整测试、发布构建和最终 0.4.3 安装均通过。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | Windows 安装版触发截图并观察窗口过渡 | pass | 用户确认系统动画已消失 |
| New / updated test | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle windows_capture_overlay_disables_show_transitions` | pass | 1 passed |
| Regression suite | `npm test`; `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 前端 42 tests；Rust 全套通过 |
| Lint / type-check | `npm run build`; `npm run lint`; `npm run format:check`; `cargo fmt -- --check` | pass | 无错误 |
| Release build/install | `npm run tauri build`; NSIS `/S` | pass | 最终安装版本 0.4.3 |

## Output Excerpts

```text
test windows_capture_overlay_disables_show_transitions ... ok
Test Files  13 passed (13)
Tests       42 passed (42)
Finished 2 bundles
ProductVersion 0.4.3
```

## Residual Risks

- 系统动画修复后暴露的黑色占位帧属于内容准备时序问题，已由独立 bug `windows-capture-overlay-black-flash` 修复。

## Recommendation

Close the bug — verified end-to-end on Windows.
