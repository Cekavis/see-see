# Quickstart: Anthropic Messages Reasoning Effort

Run the focused checks from the repository root:

```powershell
powershell.exe -Command "npm test -- src/views/Settings.model.test.tsx"
cargo test --manifest-path src-tauri/Cargo.toml --test provider_contracts
cargo test --manifest-path src-tauri/Cargo.toml --test model_config
powershell.exe -Command "npm run lint"
powershell.exe -Command "npm run format:check"
powershell.exe -Command "npm run build"
```

Manual settings check: open model settings, choose Anthropic Messages, verify the selector starts at `low`, save each supported value, reopen the card, and confirm the displayed effort matches.
