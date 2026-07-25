# Bug Verification: 模型测试后保存状态丢失

- **Slug**: model-test-save-state
- **Tested**: 2026-07-25
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

原始“输入完整信息 → 测试通过 → 再保存”流程已由自动化等价场景覆盖：API Key 在测试前保存，测试按已保存 ID 执行，通过后自动设置当前模型，后续保存继续复用同一 ID且不清除已存 Key。未发现相关回归。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | `npm test -- --run` 中的模型设置流程 | pass | 覆盖测试前保存、测试通过、自动激活和再次保存 |
| New / updated tests | `npm test -- --run` | pass | 12 个测试文件、30 项测试通过 |
| Credential persistence | `cargo test --manifest-path src-tauri/Cargo.toml --test model_config` | pass | 3 项 Key 写入、摘要读取、保留和清除测试通过 |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 授权 wiremock 回环端口后所有 Rust 测试通过 |
| Lint / type-check | `npm run lint`; `npm run build` | pass | ESLint、TypeScript 与 Vite 构建通过 |
| Formatting | `npm run format:check` | pass | 所有文件符合 Prettier 格式 |
| Release build | `npm run tauri build` | pass | 生成 See See 0.2.4 `.app` 与 aarch64 DMG |
| Local installation | 读取安装包版本并比较可执行文件 SHA-256 | pass | `/Applications/See See.app` 为 0.2.4，哈希与构建产物一致 |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests       30 passed (30)

running 3 tests
test result: ok. 3 passed; 0 failed

All matched files use Prettier code style!

Finished 2 bundles at:
  .../bundle/macos/See See.app
  .../bundle/dmg/See See_0.2.4_aarch64.dmg
```

## Residual Risks

- 验证没有使用用户的真实服务 API Key，以避免接触或泄露凭据；远端成功结果由前端测试桩提供，系统凭据写入/读取契约由 Rust 内存凭据存储测试覆盖。
- 首次对真实服务测试失败时，配置会按界面提示保留为已保存但测试失败，且不会被设为当前模型。

## Recommendation

关闭该 Bug。原始状态丢失流程已通过自动化等价场景验证，凭据持久化契约和发行安装也已分别确认。
