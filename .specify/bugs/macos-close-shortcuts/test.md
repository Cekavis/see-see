# Bug Verification: macOS window close shortcuts

- **Slug**: macos-close-shortcuts (from this task's fix report)
- **Tested**: 2026-09-18
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: verified

## Summary

The user tested the installed 0.16.1 macOS build and confirmed: “我测试了，没问题”. Native main-window behavior and the automated regression checks also passed.

## Checks Performed

| Check                     | Command / Action                                                                                    | Result     | Notes                                                                                                     |
| ------------------------- | --------------------------------------------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------- |
| Original symptom          | Command+W in the installed 0.16.0 main window                                                       | reproduced | Window remained visible.                                                                                  |
| macOS main window         | Escape, then Command+W in installed 0.16.1                                                          | pass       | Escape kept the window open; Command+W hid it.                                                            |
| macOS result window       | User tested the installed fix                                                                       | pass       | User confirmed the requested shortcuts work; desktop automation's result-window attempt was inconclusive. |
| Frontend regression suite | `npm test`                                                                                          | pass       | 15 files, 89 tests.                                                                                       |
| Rust regression suite     | `cargo test --manifest-path src-tauri/Cargo.toml`                                                   | pass       | 116 passed, two existing credential-store tests ignored.                                                  |
| Browser smoke flow        | `npm run test:e2e`                                                                                  | pass       | Primary desktop flow.                                                                                     |
| Lint and formatting       | `npm run lint`, `npm run format:check`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | pass       | Bug reports formatted separately because `.specify` is normally ignored by Prettier.                      |
| Frontend build            | `npm run build`                                                                                     | pass       | TypeScript and Vite.                                                                                      |
| Default release build     | `npm run tauri build`                                                                               | partial    | App and DMG signed; updater signing requires the unavailable private key.                                 |
| Local release build       | `npm run tauri build -- --config '{"bundle":{"createUpdaterArtifacts":false}}'`                     | pass       | Signed app and DMG; override did not modify release configuration.                                        |
| Signing continuity        | `npm run verify:macos-signature`, comparison of designated requirements                             | pass       | Stable `See See Local Release` certificate and bundle identifier.                                         |
| Local installation        | Bundle version, strict signature verification, binary SHA-256 comparison                            | pass       | `/Applications/See See.app` is 0.16.1 and matches the build.                                              |
| Windows desktop           | Manual Windows checks                                                                               | not-run    | No Windows host available; Windows implementation unchanged.                                              |

## Output Excerpts

```text
Tests 89 passed (89)
windowing::tests::macos_close_keys_respect_window_scope_and_modifiers ... ok
✓ See See primary desktop flow
签名验证通过：src-tauri/target/release/bundle/macos/See See.app
Installed version: 0.16.1
Installed/build SHA-256: 81fa847927d970265204ff8f336082fa120215b5075c0c87c5689d38a4c4a47f
User: 我测试了，没问题
```

## Residual Risks

- Windows manual regression checks were not performed.
- Updater artifacts were not signed or published; local installation is complete.

## Recommendation

Close the bug based on passing automated checks and the user's macOS verification.
