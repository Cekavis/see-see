# Bug Verification: macOS 遮罩加入其他应用全屏 Space

- **Slug**: macos-multidisplay-capture-space-switch-3
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: failed

## Summary

全屏主屏的原始症状消失，但第二屏普通桌面的遮罩出现对称回归；双显示器无法同时框选，因此修复失败。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 双屏触发 `Command+Shift+2`，查询 on-screen 与 all windows | fail | 两个 frame 正确，但只有主屏遮罩 on-screen |
| Policy test | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` | pass | 静态行为位无法证明每屏 Space 可见性 |
| Cancel | 发送 Esc | pass | 测试遮罩关闭 |

## Output Excerpts

```text
capture_windows_all=2
onscreen=1 bounds=1920×1080 @ (0,0)
onscreen=? bounds=1710×1107 @ (99,1080)
```

## Residual Risks

- 普通桌面与其他应用全屏 Space 同时存在时，单一 collection behavior 无法覆盖所有显示器。

## Recommendation

重新评估为逐窗口自适应策略；在真实双屏布局下同时验证两个 `kCGWindowIsOnscreen` 值。
