# Bug Fix: macOS 旧进程的结果窗落到 Desktop Space

- **Slug**: macos-result-existing-process-fullscreen-space
- **Fixed**: 2026-08-14
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

把 See See 以 accessory（菜单栏工具）身份运行，并把结果窗口策略改为 `CanJoinAllSpaces | FullScreenAuxiliary`。这样即使 See See 进程原本停在 Desktop Space，结果窗口也能加入用户当前所在的其他应用独占全屏 Space，而不会被拖回 Desktop Space。

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/lib.rs` | modified | 启动时 `set_activation_policy(Accessory)`。 |
| `src-tauri/src/windowing.rs` | modified | 结果窗口策略改为 `CanJoinAllSpaces | FullScreenAuxiliary`，移除 `MoveToActiveSpace` 及死代码。 |
| `src-tauri/Cargo.toml` | modified | 增加 `[profile.release] strip = "none"`，规避 macOS 27 上 Rust release 剥离 proc-macro dylib 导致的 `mis-aligned LINKEDIT string pool` 构建失败。 |
| `src-tauri/tests/desktop_lifecycle.rs` | modified | 更新结果窗口策略断言。 |
| 版本同步文件 | modified | 同步 SemVer 到 `0.10.1`。 |

## Diff Highlights

```rust
// lib.rs
#[cfg(target_os = "macos")]
app.set_activation_policy(tauri::ActivationPolicy::Accessory);
```

```rust
// windowing.rs
WindowRole::Result => WindowPolicy {
    collection_behavior: CAN_JOIN_ALL_SPACES | FULL_SCREEN_AUXILIARY,
    elevated_overlay_level: false,
},
```

## Local Verification

- `cargo test` → 全绿。
- `npm test` / `npm run build` / `npm run lint` → 通过。
- `npm run tauri -- build --bundles app` → 使用 `See See Local Release` 签名成功。
- 真机验证：在 ChatGPT / Zen 独占全屏 Space 中触发截图翻译，结果窗口出现在当前全屏 Space（用户确认通过）。
