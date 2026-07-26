# Bug Fix: macOS 自用升级使用稳定签名身份

- **Slug**: screen-permission-upgrade-identity
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

macOS 构建现在强制使用同一个 `See See Local Release` 自签名代码签名证书，并提供产物校验器阻止 ad-hoc 或 CDHash 身份回归。0.3.2 已用证书指纹 `095D7F5883674A4A3A0D219B60C69B6168C9844F` 构建并安装，其 designated requirement 固定为 bundle identifier 与证书叶指纹。

## Changes

| File | Change | Notes |
| --- | --- | --- |
| `src-tauri/tauri.macos.conf.json` | added | macOS 构建强制使用 `See See Local Release` 并启用 hardened runtime |
| `scripts/verify-macos-signature.mjs` | added | 严格验证 bundle 签名、证书、资源封装和稳定 designated requirement |
| `scripts/verify-macos-signature.test.mjs` | added test | 覆盖有效自签名身份、ad-hoc/CDHash 回归和错误证书 |
| `docs/macos-local-signing.md` | added | 记录建证、信任、构建、首次迁移、后续升级和其他设备配置 |
| `README.md` | modified | 将 macOS 自用打包入口指向签名操作文档和校验命令 |
| `package.json` / `package-lock.json` | modified | 增加签名测试/校验命令并同步 0.3.2 |
| `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock` / `src-tauri/tauri.conf.json` | modified | 同步补丁版本 0.3.2 |
| `tests/e2e/primary-flow.mjs` | modified | 同步 E2E 版本断言 |
| `src-tauri/tests/model_config.rs` | modified | 等价调整布尔断言以通过 Rust 1.97 clippy |

## Diff Highlights

macOS 平台配置声明固定签名 identity：

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "See See Local Release",
      "hardenedRuntime": true
    }
  }
}
```

真实 0.3.2 产物的身份不再以 `cdhash` 开头：

```text
designated => identifier "app.seesee.desktop" and certificate leaf = H"095d7f5883674a4a3a0d219b60c69b6168c9844f"
```

## Tests Added or Updated

- `scripts/verify-macos-signature.test.mjs` — 接受稳定自签名身份；拒绝 ad-hoc、CDHash、未封装资源和错误证书。
- `tests/e2e/primary-flow.mjs` — 验证前端读取同步后的 0.3.2 版本。
- `src-tauri/tests/model_config.rs` — 保持既有模型复制测试语义并满足当前 clippy。

## Local Verification

- `npm run test:macos-signing` → pass，3/3。
- `npm run verify:macos-signature` → pass，证书与 designated requirement 符合预期。
- `npm test` → pass，12 files / 36 tests。
- `npm run lint`; `npm run typecheck`; `npm run format:check` → pass。
- `npm run test:e2e` → pass。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → pass。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` → pass。
- `cargo test --manifest-path src-tauri/Cargo.toml` → pass，所有 Rust 单元、集成和文档测试。
- `npm run tauri build` → pass，生成签名的 `.app` 和 `See See_0.3.2_aarch64.dmg`。
- 本机安装 → pass，`/Applications/See See.app` 版本为 0.3.2，`codesign --verify --deep --strict` 通过。
- Manual checks: 用户已创建并信任固定证书；屏幕录制权限的一次性迁移与真实截图确认在验证报告中记录。

## Deviations from Assessment

- 用户明确限定为少量自用设备且不购买 Apple Developer 会员，因此采用 assessment 中的本地稳定证书替代方案，而非 Developer ID 与公证。`spctl` 按预期拒绝未公证的 app/DMG；受控设备需要信任公开证书并首次确认打开。
- 自签名证书没有 Apple TeamIdentifier；稳定身份由固定证书叶指纹和 `app.seesee.desktop` 共同组成。校验器据此拒绝单次构建 CDHash。
- Rust 1.97 clippy 在既有 `model_config` 测试中拒绝与 `true` 比较的断言；进行了等价机械修正，使仓库要求的 clippy 检查可通过。

## Follow-ups

- 安全备份 `See See Local Release` 私钥；只向运行设备分发公开 `.cer`，不要把证书材料提交仓库。
- 在第二台自用 Mac 上导入并信任 `.cer`，完成一次安装和屏幕权限迁移验证。
- 如果未来公开分发，替换为 Developer ID Application 证书并加入 Apple 公证，但保持 bundle identifier 不变。
