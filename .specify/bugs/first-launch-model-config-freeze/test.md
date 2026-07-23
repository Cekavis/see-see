# Bug Verification: 首次启动配置模型窗口空白卡死

- **Slug**: first-launch-model-config-freeze
- **Tested**: 2026-07-23
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

原始问题已在修复前稳定复现，并在开发版与本地安装的 0.1.2 发布版中消失；未发现相关测试或构建回归。

## Checks Performed

| Check | Command / Action | Result | Notes |
|-------|------------------|--------|-------|
| Reproduction (post-fix) | 首次启动状态点击“配置模型”，通过 WebView 调试目标检查新窗口 | pass | 安装版新窗口加载 `http://tauri.localhost/`，状态 `complete`，标题“模型设置” |
| New / updated tests | 真实 Tauri/WebView2 开发版与安装版回归 | pass | 修复前为 `about:blank`，修复后完整显示设置表单 |
| Frontend regression suite | `npm test` | pass | 8 个测试文件、15 个测试通过 |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 全部 Rust 测试通过 |
| Lint / type-check | `npm run lint`; `npm run build` | pass | ESLint、TypeScript 与 Vite 构建通过 |
| Formatting | `npm run format:check`; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | pass | 前端与 Rust 格式检查通过 |
| Release build | `npm run tauri build` | pass | 生成 MSI 与 NSIS 0.1.2 安装包 |
| Local install | NSIS 安装包 `/S` | pass | 退出码 0，已安装文件版本与产品版本均为 0.1.2 |

## Output Excerpts

- `Test Files  8 passed (8)` / `Tests  15 passed (15)`
- `Finished test profile`，所有 Rust 测试结果为 `ok`
- `Finished 2 bundles`，产出 `See See_0.1.2_x64_en-US.msi` 与 `See See_0.1.2_x64-setup.exe`
- 安装版回归结果：`{"targetCount":2,"heading":"模型设置","url":"http://tauri.localhost/","ready":"complete"}`

## Residual Risks

- macOS 未进行手工窗口验证；改动仅改变 Tauri 命令调度方式，现有跨平台 Rust 编译路径未改变。

## Recommendation

关闭该 Bug；Windows 首次启动流程已端到端验证。
