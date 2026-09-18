# Quickstart: WebDAV 配置同步

1. Open **常规** in See See.
2. In **WebDAV 配置**, enter the WebDAV URL, username, and password. Leave the remote root as `see-see` or enter a private relative directory.
3. Save the connection settings. The password is remembered by the operating system credential store and is never shown again.
4. Click **上传配置** on the source device.
5. On another Windows or macOS device, enter the same WebDAV settings and click **下载配置**.
6. Confirm the model and prompt lists are available. Prompt shortcuts and model API Keys remain device-local and must be configured locally.

## Validation commands

```powershell
powershell.exe -Command "npm run typecheck"
powershell.exe -Command "npm test"
cargo test --manifest-path src-tauri/Cargo.toml
powershell.exe -Command "npm run lint"
powershell.exe -Command "npm run format:check"
```

Manual UI review should cover a normal desktop settings width and a narrow window, including initial loading, saved-password state, disabled buttons during sync, success counts, and failure recovery.
