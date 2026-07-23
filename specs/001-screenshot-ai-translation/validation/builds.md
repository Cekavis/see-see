# Build validation

Date: 2026-07-23

## Environment

- OS: Windows x64
- Node.js: 25.9.0 (plan baseline is Node.js 24 LTS; current version satisfies `>=24` but is not the planned LTS line)
- npm: 11.12.1
- rustc: 1.95.0
- cargo: 1.95.0
- Chrome used by WebdriverIO: 150.0.7871.129

## Passed checks

- `npm run format:check`: passed after excluding tool/generated/spec/Rust directories and formatting frontend/root files.
- `npm run typecheck`: passed.
- `npm run lint`: passed.
- `npm test -- --run`: 8 files, 15 tests passed.
- `npm run test:e2e`: 1 WebdriverIO Chrome IPC smoke passed.
- `npm run build`: passed; production JS bundle about 235 KiB (about 72 KiB gzip).
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -j 1 -- -D warnings`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml -j 1`: passed, 32 tests total including the new runtime-state regression test.
- `cargo test --release --manifest-path src-tauri/Cargo.toml --test history_benchmark`: passed previously; 10,000-record benchmark body completed in about 0.03 seconds after cached compilation.
- `npm run tauri dev`: dev build compiled and launched `see-see.exe`.
- `npm run tauri build`: passed after declaring the existing icon files in `tauri.conf.json`.

## Windows artifacts

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/see-see.exe` | 22,361,088 | `9A61303C65A6D8C4CB64F8D4053683503700D2F4106846084B11F7B44039A0EC` |
| `src-tauri/target/release/bundle/msi/See See_0.1.0_x64_en-US.msi` | 7,847,936 | `190A33570DFD4A78618C9A51BF26522CE9C25735B060F8CF191710D38ACEC1EB` |
| `src-tauri/target/release/bundle/nsis/See See_0.1.0_x64-setup.exe` | 5,327,257 | `92AE9B1B8F885380732F0D40B913FAC27BD74745AFB07FB96B6D504E08092EDA` |

## Known failures and gaps

- The first Tauri bundle attempt failed because `bundle.icon` was empty even though icon files existed. The config now declares the existing PNG, ICNS, and ICO files; the rerun produced both installers.
- `@wdio/tauri-service` 1.2.0 could not initialize because it imports `installMockSyncOverride`, absent from its pinned `@wdio/native-utils` 2.4.0 and current 2.5.0 package exports. The unusable service was removed; WebdriverIO now uses native Chrome with a narrow pre-load IPC mock bridge, while actual desktop startup is verified separately by `tauri dev`.
- `npm install` reported 3 dependency advisories (2 moderate, 1 high). Detailed audit was not authorized because it transmits dependency metadata to npm.
- No macOS command or build was run. T071 remains open until the same checks and a universal Tauri build pass on macOS 14+.
