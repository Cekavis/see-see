# Quickstart: Tray Click Settings

## Automated validation

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm test
npm run lint
npm run format:check
npm run build
```

## Desktop interaction validation

1. Start the packaged or development application and hide the settings window.
2. Left-click the tray icon once.
3. Confirm the existing settings window is restored and focused and the tray menu is not shown.
4. Right-click the tray icon.
5. Confirm the existing tray menu appears with capture, open, and quit actions.
6. Repeat with the settings window minimized and already visible to confirm no duplicate window is created.

Perform the interaction check on Windows and macOS because tray events are platform integrations.

## Verification record

- Automated checks: Passed on Windows on 2026-07-31.
- Release bundle: MSI and NSIS packages built successfully for version 0.5.0.
- Local installation: NSIS version 0.5.0 installed and the settings window launched successfully on Windows.
- Windows tray interaction: Not manually verified because the automation environment could not target the taskbar notification area.
- macOS tray interaction: Not verified because no macOS environment was available.
