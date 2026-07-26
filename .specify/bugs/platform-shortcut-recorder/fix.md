# Bug Fix: 平台默认快捷键与快捷键录入器

- **Slug**: platform-shortcut-recorder
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

截图快捷键现在使用明确的 Windows/macOS 默认值，并安全迁移历史默认值。桌面设置中的自由文本框已替换为快捷键录入按钮，点击后按组合键即可自动注册、保存并立即生效。

## Changes

| File | Change | Notes |
| --- | --- | --- |
| `src-tauri/src/database.rs` | modified | 定义 Windows/macOS 默认值，数据库版本升至 4，并只迁移旧默认值 |
| `src-tauri/tests/desktop_lifecycle.rs` | updated test | 验证当前平台的新数据库默认值 |
| `src/views/DesktopSettings.tsx` | modified | 捕获并规范化键盘组合，自动调用快捷键注册 IPC，失败时回退 |
| `src/views/Settings.desktop.test.tsx` | updated tests | 覆盖 macOS Command 录入、单独修饰键忽略和冲突回退 |
| `src/styles.css` | modified | 增加快捷键录入与录制态样式 |
| `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json` | modified | 同步补丁版本至 `0.3.1` |

## Diff Highlights

- Windows 默认值为 `Ctrl+Shift+X`，macOS 默认值为 `Command+Shift+X`。
- `user_version < 4` 时，仅当保存值仍为历史默认 `Alt+Shift+A` 才迁移，用户自定义值保持不变。
- 录入控件使用 `KeyboardEvent.code` 生成 Tauri/global-hotkey 可解析的键名；字母与数字以易读形式保存。
- 新组合注册期间禁用重复输入；注册冲突时继续保留旧组合。

## Tests Added or Updated

- `database::tests::platform_defaults_are_explicit` — 锁定两个平台各自的默认组合。
- `database::tests::legacy_default_is_migrated_for_the_current_platform` — 锁定历史默认迁移。
- `database::tests::custom_shortcut_is_preserved_during_platform_migration` — 防止覆盖用户自定义值。
- `DesktopSettings > captures a shortcut and applies it immediately` — 验证 Command 组合自动应用。
- `DesktopSettings > keeps the old shortcut when the captured combination conflicts` — 验证冲突回退。

## Local Verification

- Commands run: `npm test -- --run src/views/Settings.desktop.test.tsx` → 3 tests passed.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml database::tests` → 3 tests passed.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml desktop_settings_only_persist_after_system_success` → 1 test passed.
- Commands run: `npm run typecheck` → passed.
- Manual checks: 尚未在打包后的 macOS 应用中执行真实全局注册交互；留待完整构建与本机安装验证。

## Deviations from Assessment

没有单独新增 SQL 迁移文件；项目现有数据库初始化器已集中管理兼容升级，因此平台默认迁移直接接入该版本门控流程。

## Follow-ups

- 在真实 macOS WebView 中点击录入控件并录入一个未被系统占用的 Command 组合，确认全局触发截图。
- Windows 发布前验证 `Ctrl+Shift+X` 首次安装默认值与录入交互。
