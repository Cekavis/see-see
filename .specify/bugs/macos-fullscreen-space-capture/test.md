# Bug Verification: macOS 当前 Space 全屏截图

- **Slug**: macos-fullscreen-space-capture
- **Tested**: 2026-07-26
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

窗口策略、紧凑尺寸、完整自动化、签名发布构建和本机 0.3.4 安装均通过，未发现相关回归。安装后的原生应用可以正常启动，但当前 GUI 自动化通道不能发送系统级全局快捷键，且主窗隐藏后无法操作菜单栏状态项，因此尚未实际完成系统全屏 Space 中的原始手工复现步骤，不能标记为完全 verified。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 安装 0.3.4，在普通窗口及系统全屏窗口所在 Space 按全局快捷键 | not-run | Computer Use 可启动并读取 See See，但不支持系统级全局快捷键；隐藏主窗后也无法访问菜单栏状态项 |
| New / updated tests | `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_lifecycle` | pass | 6 passed；覆盖互斥 Space 策略、全屏辅助标志和 `460×500` / `420×360` 尺寸合同 |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 全部单元、集成及文档测试通过；mock server 测试允许本机回环监听后通过 |
| Frontend regression suite | `npm test` | pass | 单独运行 12 files / 37 tests 全部通过 |
| Browser smoke flow | `npm run test:e2e` | pass | `See See primary desktop flow` |
| Lint / type-check | `npm run lint`; `npm run build`; `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | pass | ESLint、TypeScript/Vite 构建和 Clippy 均通过 |
| Formatting | `npm run format:check`; `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | pass | Prettier 与 rustfmt 通过 |
| Release build | `npm run tauri build` | pass | 生成签名的 `See See_0.3.4_aarch64.dmg` 与应用包 |
| Signature | `npm run verify:macos-signature` | pass | 证书为 `See See Local Release`，designated requirement 保持稳定 |
| Local installation | 安装 `/Applications/See See.app`，比较版本、签名和 SHA-256 | pass | 安装版本为 0.3.4，安装后二进制与构建产物哈希一致 |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests  37 passed (37)
```

```text
test result: ok. 6 passed; 0 failed
✓ See See primary desktop flow
签名验证通过：src-tauri/target/release/bundle/macos/See See.app
```

```text
0.3.4
07d2f2d2795ecf70da8bfcc8cf9a5f52f8f35a20ddc41e359a60e5497c2173cc  /Applications/See See.app/Contents/MacOS/see-see
07d2f2d2795ecf70da8bfcc8cf9a5f52f8f35a20ddc41e359a60e5497c2173cc  src-tauri/target/release/bundle/macos/See See.app/Contents/MacOS/see-see
```

## Residual Risks

- 仍需在 macOS 26+ 的另一个应用系统全屏 Space 中手动按一次当前全局快捷键，确认遮罩不切换 Space，框选后的 `460×500` 结果窗留在同一 Space。
- 多显示器的 Space 组合、Esc 关闭全部遮罩和跨屏选区继续需要人工复验；自动化已覆盖坐标合成与窗口策略合同，但不能替代 AppKit 实际呈现。
- 前端测试与两个 Rust 全量编译并行时曾有 3 个异步测试超时；脱离资源竞争后 37/37 通过，判断为验证环境争用而非产品回归。

## Recommendation

保持修复并发布 0.3.4；请在已安装版本中完成一次普通窗口和一次系统全屏窗口的快捷键截图。两项均留在当前 Space 后即可关闭该 Bug；若仍发生 Space 跳转，则携带实际窗口/显示器布局重新运行 `/speckit-bug-assess slug=macos-fullscreen-space-capture`。
