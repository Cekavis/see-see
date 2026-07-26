# Bug Fix: Keep the result copy button visible

- **Slug**: result-copy-button-clipped
- **Fixed**: 2026-07-26
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

The result view now uses one constrained flexible row for recognition text, so long output scrolls inside the text area while the footer and copy button remain fully visible.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `src/styles.css` | modified | Corrected the result grid from four rows to the three rows rendered by the component. |
| `src/views/Result.test.tsx` | added test | Pins the grid-row contract that keeps the footer outside the scrolling content area. |
| `package.json` | modified | Bumped the patch version to 0.3.6. |
| `package-lock.json` | modified | Synchronized the package lock version. |
| `src-tauri/Cargo.toml` | modified | Bumped the Rust package version to 0.3.6. |
| `src-tauri/Cargo.lock` | modified | Synchronized the Rust package lock version. |
| `src-tauri/tauri.conf.json` | modified | Bumped the Tauri bundle version to 0.3.6. |

## Tests Added or Updated

- `src/views/Result.test.tsx` — verifies `.result-view` declares `auto minmax(0, 1fr) auto`, assigning the recognition text to the only flexible row.

## Local Verification

- Commands run: `npm test -- src/views/Result.test.tsx` → passed, 3 tests.
- Commands run: `npm run typecheck` → passed.
- Commands run: `npm run lint` → passed.
- Commands run: `npm run format:check` → passed.
- Manual checks: source layout and the supplied 460 × 500 result-window screenshot were compared; packaged-app verification follows in the test phase.

## Deviations from Assessment

None.

## Follow-ups

- Verify the packaged result window with a long result after local installation.
