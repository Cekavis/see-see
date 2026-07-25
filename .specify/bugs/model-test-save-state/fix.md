# Bug Fix: 模型测试后保存状态丢失

- **Slug**: model-test-save-state
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

模型连接测试现在会先持久化完整配置和 API Key，再使用保存后的 ID 测试；通过后配置会自动设为当前模型。普通保存也会将返回 ID 写回表单，避免后续操作创建重复配置或无法保留测试状态。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src/views/Settings.tsx` | modified | 串联保存、凭据读取、连接测试与激活流程，并阻止模型操作并发 |
| `src/views/Settings.model.test.tsx` | updated tests | 覆盖测试前保存 Key、按 ID 测试、成功激活、失败不激活和保存后复用 ID |
| `package.json` | modified | 补丁版本提升到 0.2.4 |
| `package-lock.json` | modified | 同步 npm 根包版本 |
| `src-tauri/Cargo.toml` | modified | 同步 Rust 包版本 |
| `src-tauri/Cargo.lock` | modified | 同步 Rust 锁文件根包版本 |
| `src-tauri/tauri.conf.json` | modified | 同步 Tauri 应用版本 |

## Diff Highlights

“测试连接”先调用 `saveModelConfig`，再以返回摘要构造不包含明文 Key、但包含稳定配置 ID 的测试请求。后端通过该 ID 从系统凭据存储读取 Key并持久化测试结果；通过后调用既有的 `setActiveModelConfig` 约束设置当前模型。

## Tests Added or Updated

- `src/views/Settings.model.test.tsx` — 验证新配置的 Key 先保存，测试请求携带保存后的 ID且不回传明文 Key，测试通过后自动激活。
- `src/views/Settings.model.test.tsx` — 验证失败测试不会激活模型。
- `src/views/Settings.model.test.tsx` — 验证保存返回 ID 写回表单，后续保存复用同一配置并保留已存 Key。

## Local Verification

- Commands run: `npm test -- --run` → 12 个测试文件、30 项测试通过（添加最终精确流程断言前）。
- Commands run: `npm run lint` → 通过。
- Commands run: `npm run format:check` → 格式修正后通过。
- Commands run: `npm run build` → TypeScript 检查与 Vite 构建通过。
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml` → 受限环境首次无法绑定 wiremock 回环端口；授权本机回环端口后全套 Rust 测试通过。
- Commands run: `npm run tauri build` → 授权 macOS DMG 工具后成功生成 `.app` 与 `See See_0.2.4_aarch64.dmg`。
- Manual checks: 已将生成的 See See 0.2.4 应用安装到 `/Applications/See See.app`。

## Deviations from Assessment

无。

## Follow-ups

- 在真实服务凭据下确认连接测试后配置卡片显示“已保存 Key · 测试通过”；自动化测试使用内存凭据存储和模拟连接结果，不接触真实 API Key。
