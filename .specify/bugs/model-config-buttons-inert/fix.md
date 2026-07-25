# Bug Fix: 模型配置按钮缺少可见反馈

- **Slug**: model-config-buttons-inert
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

将三个模型操作按钮的反馈移动到编辑器动作区，并增加明确的进行中文案与互斥状态，使点击后的进度和结果始终可见。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src/views/Settings.tsx` | modified | 操作反馈移入编辑器；新增统一动作启动逻辑、进行中文案和并发禁用 |
| `src/views/Settings.model.test.tsx` | added regression coverage | 验证反馈位置、三个进行中状态及并发禁用 |
| `src/styles.css` | modified | 为动作反馈预留稳定空间，防止消息出现时把按钮推出视口 |
| `package.json` / `package-lock.json` | modified | 补丁版本升级到 0.2.2 |
| `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock` / `src-tauri/tauri.conf.json` | modified | 同步应用版本 0.2.2 |

## Diff Highlights

三个按钮现在在请求期间分别显示“正在获取…”“正在测试…”和“正在保存…”。新动作会清除旧消息，且任一动作进行中时禁止同时启动其他模型操作。

## Tests Added or Updated

- `src/views/Settings.model.test.tsx` — 连接失败反馈必须位于“模型配置编辑器”区域内。
- `src/views/Settings.model.test.tsx` — 三个异步操作均显示即时进度，并在完成前禁用所有模型操作按钮。

## Local Verification

- Commands run: `npm test -- --run src/views/Settings.model.test.tsx` → 4 个测试通过。
- Commands run: `npm run typecheck` → 通过。
- Commands run: `npm run lint` → 通过。
- Commands run: `npm run format:check` → 通过。
- Manual checks: 浏览器中点击“保存配置”后，反馈位于编辑器内 `top: 550.8–595.8px`，按钮位于 `621.8–657.8px`，不再与粘性标题栏 `0–88px` 重叠。

## Deviations from Assessment

无。

## Follow-ups

- 在 macOS 和 Windows 安装包中各手动确认一次真实 Tauri IPC 的成功与失败反馈。
