# Bug Verification: macOS 稳定签名身份与权限迁移

- **Slug**: screen-permission-upgrade-identity
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

自签名 0.3.2 已获得稳定的证书 + bundle identifier 身份，严格签名校验和首次权限迁移通过；用户确认应用显示“屏幕权限已就绪”。但真实截图触发暴露出另一个 macOS 27 兼容性缺陷：现有 xcap 后端仍调用 macOS 15 起 obsolete 的 CoreGraphics 截图 API，因此本次验证不能标记为完全通过。

## Checks Performed

| Check | Command / Action | Result | Notes |
| --- | --- | --- | --- |
| Reproduction (post-fix) | 安装签名 0.3.2、删除旧 ad-hoc TCC 条目、添加当前 app 并重启 | pass | 用户确认“权限已就绪”，`CGPreflightScreenCaptureAccess()` 对当前身份返回已授权 |
| Stable identity | `codesign -d --requirements - "/Applications/See See.app"` | pass | requirement 为 `app.seesee.desktop` + 固定证书叶指纹，不再是 CDHash |
| App/DMG integrity | `codesign --verify --deep --strict` | pass | `.app` 和 `.dmg` 均 valid on disk 且满足 designated requirement |
| New signing tests | `npm run test:macos-signing` | pass | 3/3 |
| Frontend regression | `npm test` | pass | 12 files / 36 tests |
| Rust regression | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 所有 Rust 单元、集成和文档测试通过 |
| Browser regression | `npm run test:e2e` | pass | 主桌面流程通过 |
| Lint / type-check / format | `npm run lint`; `npm run typecheck`; `npm run format:check`; cargo fmt/clippy | pass | 无检查错误 |
| Release bundle | `npm run tauri build` | pass | 生成签名 0.3.2 app 和 DMG |
| Real capture | 快捷键与菜单栏“开始截图” | fail | 两个入口均记录 `capture_failed`，详见 `../macos-capture-api-obsolete/assessment.md` |

## Output Excerpts

```text
Designated requirement：identifier "app.seesee.desktop" and certificate leaf = H"095d7f5883674a4a3a0d219b60c69b6168c9844f"

/Applications/See See.app: valid on disk
/Applications/See See.app: satisfies its Designated Requirement

Test Files  12 passed (12)
Tests       36 passed (36)
```

用户确认：

```text
权限已就绪，但是截图无法触发，包括我手动从菜单栏里点截图也没有反应
```

## Residual Risks

- 尚未用第二个同证书签名版本覆盖 0.3.2，跨两个稳定签名版本的 TCC 继承仍需下一次升级复验。
- 自签名包不受 Apple 公证，Gatekeeper 仍会拒绝自动放行；受控设备需要信任证书并首次确认打开。
- macOS 27 的真实截图失败是独立的捕获后端兼容性问题，在修复前应用核心截图功能不可用。

## Recommendation

保留稳定签名修复；权限身份与权限检测链路已按预期工作。暂不关闭整体发布验证，先修复 `macos-capture-api-obsolete`，用 ScreenCaptureKit 完成真实截图后再验证下一次同证书覆盖升级。
