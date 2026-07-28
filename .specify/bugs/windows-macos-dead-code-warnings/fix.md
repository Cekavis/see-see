# Bug Fix: Windows build warns about macOS-only capture helpers

- **Slug**: windows-macos-dead-code-warnings
- **Fixed**: 2026-07-28
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Restricted three macOS capture helpers to macOS production builds or test builds, removing Windows `dead_code` warnings without suppressing the lint. Added a Git-native LF rule and normalized the current tracked text worktree for consistent Windows/macOS development.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src-tauri/src/capture.rs` | modified | Added `cfg(any(target_os = "macos", test))` to the three macOS/test-only definitions. |
| `.gitattributes` | added | Enforces LF for detected text files across platforms. |

## Diff Highlights

```rust
#[cfg(any(target_os = "macos", test))]
```

```gitattributes
* text=auto eol=lf
```

## Tests Added or Updated

- No tests changed; existing permission mapping and BGRA adapter tests continue to compile these helpers under `cfg(test)`.

## Local Verification

- Commands run: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` → passed.
- Commands run: `cargo check --manifest-path src-tauri/Cargo.toml` → passed with no warnings.
- Commands run: `cargo test --manifest-path src-tauri/Cargo.toml` → passed, all Rust tests successful with no warnings.
- Commands run: `npm run lint` → passed.
- Commands run: `npm run format:check` → passed after LF normalization.
- Commands run: `npm run tauri build` → passed; Windows MSI and NSIS bundles generated with no Rust warnings.
- Manual checks: `git ls-files --eol` reports no tracked files with CRLF or mixed working-tree line endings.

## Deviations from Assessment

- The user expanded the request during implementation to standardize line endings for Windows/macOS collaboration. Added `.gitattributes` and normalized tracked text files; normalized files whose content was already LF in Git produced no repository diff.

## Follow-ups

- Run the normal macOS build in macOS CI or on a macOS development machine; only `x86_64-pc-windows-msvc` is installed locally.
