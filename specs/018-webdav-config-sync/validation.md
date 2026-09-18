# WebDAV configuration sync validation

Date: 2026-09-18. Application version: 0.16.0. Platform exercised: Windows x64.

## Automated checks

| Check                                                                                                                            | Result                                                                                                                                |
| -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `npm test`                                                                                                                       | 15 files, 87 tests passed                                                                                                             |
| `cargo test --manifest-path src-tauri/Cargo.toml`                                                                                | 114 tests passed; two opt-in native credential tests ignored by default                                                               |
| `cargo test --manifest-path src-tauri/Cargo.toml --test native_credentials password_survives_independent_processes -- --ignored` | Passed on Windows; separate processes wrote/read a temporary credential and removed it afterward                                      |
| `npm run test:e2e`                                                                                                               | Passed; includes saving WebDAV settings, upload, download cancellation/confirmation, and reopening saved fields with no password echo |
| `npm run lint`                                                                                                                   | Passed                                                                                                                                |
| `npm run build`                                                                                                                  | TypeScript and Vite production build passed                                                                                           |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`                                                                      | Passed                                                                                                                                |
| Prettier on changed frontend files, package metadata, and the browser test                                                       | Passed                                                                                                                                |
| `git diff --check`                                                                                                               | Passed                                                                                                                                |

All npm commands ran through an external PowerShell process as required by the repository.

The 13 configuration integration tests cover API Key serialization and restoration, explicit null clearing, rejection of a missing API Key field, ID/name matching, name swaps, ambiguous matches, local-only row retention, local shortcut preservation, transaction rollback, settings reopening, password whitespace, and credential rollback when SQLite fails.

Seven HTTP integration tests use a local mock WebDAV service to exercise nested MKCOL paths, PUT/GET, Basic authentication, Unicode path encoding, authentication and directory errors, missing files, redirect rejection, and the 4 MiB payload bound. No real model credentials or WebDAV accounts were read or transmitted during testing.

The existing desktop lifecycle test now expects four uses of the shared non-streaming HTTP client: model listing/testing plus WebDAV upload/download. Its assertions remain enabled. An earlier run hit the existing long-first-token streaming test intermittently; the focused rerun and final full Rust run passed.

## UI checks

Browser inspection used a local fixture with fictional connection data at 1024 × 720 and 720 × 520. Verified labelled fields, saved-password masking, custom remote-root saving, disabled controls while uploading/downloading, download merge confirmation, success recovery, and no horizontal overflow. Existing shared components and responsive styles are reused without additional CSS.

Local screenshot evidence (ignored build output):

- `build/webdav-desktop.png`
- `build/webdav-compact.png`
- `build/webdav-download-confirmation.png`

The preview browser tab and temporary Vite server were closed after inspection. Human visual acceptance remains pending; the browser review is not recorded as human review.

## Build and local installation

`npm run tauri build` completed using the existing local signing credential without printing it. It produced Windows x64 MSI and NSIS installers and both updater signatures:

- `src-tauri/target/release/bundle/msi/See See_0.16.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/See See_0.16.0_x64-setup.exe`

The NSIS package installed locally with exit code 0. The installed executable reports 0.16.0 and matches the built release executable byte-for-byte after accounting for Tauri's documented-in-source bundle marker (`NSS` in the installed NSIS binary, `UNK` in the restored build binary). The installed app launched successfully. Windows UI Automation found the native app's WebDAV address, remote-root, upload, and download controls. No user settings were changed during this smoke check.

## Existing unrelated check failures

- Repository-wide `npm run format:check` reports only the unchanged `AGENTS.md`; every changed frontend file passes its formatting check.
- `npm run test:release-config` fails because its unchanged verifier expects `uploadUpdaterJson: true`, whereas the unchanged release workflow uses `false` and generates `latest.json` separately. Neither file is part of this feature change.

## Remaining acceptance gaps

- No macOS application build or native Keychain runtime test was run. The accessible Mac lacked a discoverable Node/Rust toolchain. The same JSON format and merge implementation are used for Windows/macOS, `apple-native` is enabled, and regression fixtures exercise both platforms' shortcut strings without synchronizing them.
- No live remote WebDAV account was supplied. Real service compatibility, Windows-to-macOS device transfer, and the 10-second healthy-network performance target remain unverified; protocol tests use a local HTTP mock server.
- Human visual acceptance at representative window sizes remains pending.
