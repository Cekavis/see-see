# Bug Fix: macOS 快捷键窗口捕获与辅助文案精简

- **Slug**: shortcut-recorder-focus-copy
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

快捷键录制改为在录制期间从窗口捕获键盘事件，不再依赖 macOS WebView 是否把焦点交给按钮，并增加无 `code` 事件的键名回退。界面辅助文案已全面复查，删除一处同义复述，并在仓库规范中加入可执行的文案原则。

## Changes

| File | Change | Notes |
| --- | --- | --- |
| `src/views/DesktopSettings.tsx` | modified | 临时窗口捕获监听、`key` 回退、Esc 取消与可靠清理 |
| `src/views/Settings.desktop.test.tsx` | updated tests | 从窗口派发按键，覆盖无 `code`、修饰键、冲突、Esc 清理与文案 |
| `src-tauri/tests/storage_foundation.rs` | updated test | 同步数据库快捷键迁移后的 schema 版本断言 |
| `tests/e2e/primary-flow.mjs` | modified | 在应用脚本运行前注入 Tauri mock，避免初始化错误通知遮挡交互 |
| `AGENTS.md` | modified | 禁止仅复述相邻标题/控件的辅助文案 |
| `.specify/bugs/platform-shortcut-recorder/test.md` | added | 记录首版修复的真实 macOS 验证失败 |

## Diff Highlights

- `window.addEventListener("keydown", ..., true)` 只在录制态存在，结束或卸载时自动移除。
- `KeyboardEvent.code` 无法识别时，使用 `key` 转换字母、数字、方向键及常见标点。
- 再次点击录入器或按 Esc 可取消录制。
- 删除“登录系统后自动启动 See See。”；保留的提示均提供动态状态、隐私/存储范围、费用、约束、回退或不可逆后果等独立信息。

## Tests Added or Updated

- `DesktopSettings > captures a shortcut and applies it immediately` — 从 `window` 录入 `Command+Shift+K`，且以空 `code` 验证 `key` 回退。
- `DesktopSettings > keeps the old shortcut when the captured combination conflicts` — 窗口事件冲突后恢复旧值。
- `DesktopSettings > cancels recording with Escape and removes the window listener` — 取消后窗口按键不再触发保存。
- `DesktopSettings > syncs autostart, history preference, and exports sanitized logs` — 重复副标题不再渲染，隐私信息仍保留。
- `database_defaults_and_pragmas_match_the_plan` — 断言数据库版本 4。
- `tests/e2e/primary-flow.mjs` — 使用 macOS 默认快捷键和版本 `0.3.1`，并覆盖无初始化错误的主流程。

## Local Verification

- Commands run: `npm test -- --run src/views/Settings.desktop.test.tsx` → 4 tests passed.
- Commands run: `npm run typecheck` → passed.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml` → 首轮暴露旧 schema 版本断言，已同步为版本 4，等待重跑。
- Manual checks: 待重新构建并安装 `0.3.1` 后在真实 macOS WebView 中复现原步骤。

## Deviations from Assessment

未新增独立 copy 测试文件；文案断言与同一设置组件的交互测试放在一起，避免拆散同一行为上下文。完整验证还发现 E2E mock 注入晚于 React 初始化，导致错误通知遮挡点击；已扩展范围修复该测试基础设施。

## Follow-ups

- Windows 发布前仍需人工确认 `Ctrl+Shift+X` 默认值与窗口级录入交互。
