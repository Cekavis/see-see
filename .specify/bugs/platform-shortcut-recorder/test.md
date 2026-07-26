# Bug Verification: 平台默认快捷键与首版快捷键录入器

- **Slug**: platform-shortcut-recorder
- **Tested**: 2026-07-25
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: failed

## Summary

平台默认值与自动化测试通过，但用户在真实 macOS 应用中复现了录入器无响应：点击后进入“请按新的快捷键…”状态，随后按键不会保存。首版修复依赖按钮获得键盘焦点，未解决原始交互问题。

## Checks Performed

| Check | Command / Action | Result | Notes |
| --- | --- | --- | --- |
| Reproduction (post-fix) | 用户在 macOS 应用中点击录入器并按新组合键 | fail | 录制状态出现，但任何按键都没有反应 |
| New / updated tests | `npm test -- --run src/views/Settings.desktop.test.tsx` | pass | 测试错误地把 `keydown` 直接派发给按钮 |
| Regression suite | `npm test` | pass | 12 files / 35 tests passed |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | partial | 本次相关测试通过；2 个 wiremock 测试因沙箱禁止端口绑定未运行完成 |
| Lint / type-check | `npm run lint`; `npm run build` | pass | lint、类型检查与前端生产构建通过 |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests  35 passed (35)
```

```text
用户实测：点击之后显示“请按新的快捷键”，但按什么都没反应。
```

## Residual Risks

- 按钮级键盘事件测试无法模拟 macOS WebView 的实际焦点行为。

## Recommendation

重新评估并修复事件监听边界；后续工作记录在 `../shortcut-recorder-focus-copy/`。
