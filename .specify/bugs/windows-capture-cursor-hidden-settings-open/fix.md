# Bug Fix: 设置窗口开启时截图十字光标不可见

- **Slug**: windows-capture-cursor-hidden-settings-open
- **Fixed**: 2026-08-04
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

截图遮罩不再依赖 Windows 原生 cursor；它现在隐藏原生 cursor，并用 overlay 内的 DOM 元素绘制和移动十字光标。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src/views/CaptureOverlay.tsx` | modified | pointer move 时更新应用内十字光标，离开 overlay 时隐藏 |
| `src/styles.css` | modified | 隐藏原生 cursor 并绘制高对比度 DOM 十字 |
| `src/views/CaptureOverlay.test.tsx` | updated test | 验证十字光标显示、坐标和离开隐藏 |

## Diff Highlights

```tsx
cursor.current.hidden = false;
cursor.current.style.left = `${event.clientX}px`;
cursor.current.style.top = `${event.clientY}px`;
```

## Tests Added or Updated

- `src/views/CaptureOverlay.test.tsx` — 首次 pointer move 后光标从隐藏变为可见并位于 `clientX/clientY`，pointer leave 后再次隐藏。

## Local Verification

- `npm test -- --run src/views/CaptureOverlay.test.tsx` → passed，2 tests。
- `npm test` → passed，前端 42 tests。
- `npm run build`、`npm run lint`、`npm run format:check` → passed。
- `cargo test --manifest-path src-tauri/Cargo.toml` → passed，完整 Rust 测试通过。
- `npm run tauri build` → passed，重新生成 0.5.1 MSI 与 NSIS。
- NSIS `/S` 覆盖安装 → passed；安装文件时间更新为 2026-08-04 14:04:00，且没有旧 See See 进程运行。

## Deviations from Assessment

无。

## Follow-ups

- 打开 See See 设置窗口，在 Lossless Scaling 场景再次截图，确认应用内十字始终可见。
