# Quickstart: Validate System Proxy Support

## Automated configuration check

```powershell
npm run test:release-config
```

Expected: `release updater configuration is complete` and no assertion failure.

## Resolved dependency features

```powershell
cargo tree --manifest-path src-tauri/Cargo.toml -e features -i reqwest@0.12.28
cargo tree --manifest-path src-tauri/Cargo.toml -e features -i reqwest@0.13.4
```

Expected: both trees contain `reqwest feature "system-proxy"`.

## Regression suite

```powershell
npm run format:check
npm run lint
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
npm run test:release-config
npm run tauri build
```

Expected: all commands pass and signed Windows installers plus updater artifacts are generated.

## Manual platform validation

1. Clear proxy environment variables for the launched application.
2. Configure a reachable operating-system proxy on Windows or macOS.
3. Launch See See after the proxy is configured.
4. List models, test a model configuration, and run screenshot analysis.
5. Check for an application update.
6. Confirm the proxy observes the requests and all operations complete.
7. Disable the system proxy, relaunch See See, and confirm direct model requests still work.

## Release verification

- Release workflow `32613863665` completed successfully on 2026-08-23.
- `v0.11.0` was published as the latest release.
- The published release contains `latest.json`, signed Windows MSI and NSIS updater assets, Apple Silicon and Intel macOS DMG/app updater assets, and source archives.
