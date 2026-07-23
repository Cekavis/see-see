# Quickstart: Validate the Unified Settings Window

## Automated checks

```powershell
npm run typecheck
npm run lint
npm run format:check
npm test
npm run test:e2e
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## Desktop lifecycle

1. Launch See See and confirm the main window opens on 常规.
2. Switch through 模型、提示词、历史、关于 and confirm no new management windows appear.
3. Close the main window and use 托盘 → 打开 See See; confirm the same window returns.
4. Use 托盘 → 开始截图, then repeat with the global shortcut.
5. Choose 托盘 → 退出 and confirm the process exits.

## Visual and accessibility review

- Capture evidence at 1024×720 and 1440×900 in light and dark themes.
- Verify 200% system text scaling with no inaccessible controls or horizontal content loss.
- Verify keyboard order, visible focus, switches, disabled states, errors, empty states and confirmation dialogs.
- Verify the result window at 360×240 and the capture overlay on a real Windows desktop.
- Install and launch the generated Windows bundle before committing.

## Verification record — July 23, 2026

- Frontend formatting, TypeScript, ESLint, 20 Vitest tests and the production Vite build passed.
- WebdriverIO passed with all five local sections, no window capture control, no `open_view` calls, a 1440×900 fixed sidebar, and a 720-pixel compact top navigation without page overflow.
- Rust formatting, Clippy with warnings denied, and the complete Rust test suite passed.
- Tauri produced both MSI and NSIS bundles for version 0.2.0. The NSIS per-user installer completed successfully; the installed executable reports file and product version 0.2.0 and remains running after launch.
- Direct browser screenshot capture of the localhost preview was blocked by the current browser safety policy. Light/dark screenshots, 200% operating-system text scaling, tray capture, the global shortcut, and the 360×240 result window remain manual desktop checks and are not claimed as directly observed here.
