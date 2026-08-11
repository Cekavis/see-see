# Bug Fix: 历史页不会自动显示新记录

- **Slug**: history-auto-refresh
- **Fixed**: 2026-08-11
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

历史记录保存成功后会发送全局更新事件；已打开的历史页订阅该事件并按当前筛选条件重新查询，因此无需切换栏目即可显示新内容。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/analysis.rs` | modified | 成功或失败分析写入历史数据库后发送 `history-updated` |
| `src/views/History.tsx` | modified | 订阅更新事件、刷新第一页并在卸载时解除监听 |
| `src/views/History.test.tsx` | added test | 验证事件触发刷新和监听器清理 |
| `src/views/SettingsShell.test.tsx` | modified | 为包含历史页的壳层测试模拟 Tauri 事件 API |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | 同步补丁版本为 `0.9.1` |

## Tests Added or Updated

- `src/views/History.test.tsx` — 验证空历史页收到 `history-updated` 后重新查询并显示新记录，卸载时移除监听器。
- `src/views/SettingsShell.test.tsx` — 为设置壳层提供事件监听 mock，避免非 Tauri 测试环境产生未处理异常。

## Local Verification

- `npm run format:check` → 通过。
- `npm run lint` → 通过。
- `npm test` → 14 个文件、56 个测试全部通过，无未处理错误。
- `npm run build` → 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` → 全部通过。
- `npm run tauri build` → 通过；生成 0.9.1 MSI、NSIS 和两个 updater 签名。
- NSIS `/S` 覆盖安装 → 通过；注册表、文件和产品版本均为 0.9.1，安装版可正常以隐藏自启动模式运行。

## Deviations from Assessment

- 增加修改 `src/views/SettingsShell.test.tsx`：完整测试发现该测试会间接渲染历史页，需要模拟新增的 Tauri 事件 API。生产代码修复方案未改变。

## Follow-ups

- 无。
