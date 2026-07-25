# Bug Fix: npm development dependency vulnerabilities

- **Slug**: npm-audit-vulnerabilities
- **Fixed**: 2026-07-25
- **Assessment**: ./assessment.md
- **Status**: applied

## Summary

Removed the vulnerable WebdriverIO/Mocha development dependency graph and ported the existing browser smoke flow to a direct `playwright-core` runner. The regenerated dependency graph now reports zero npm audit findings while preserving the original E2E coverage.

## Changes

| File | Change | Notes |
|------|--------|-------|
| `package.json` | modified | Replaced five `@wdio/*` packages with `playwright-core`, added the browser-install script, and bumped the app to 0.2.1. |
| `package-lock.json` | modified | Regenerated the lockfile, removed the vulnerable WebdriverIO/Mocha/Archiver graph, and updated `brace-expansion` to 5.0.8. |
| `tests/e2e/run.mjs` | modified | Replaced the WebdriverIO launcher with a direct headless Chromium runner. |
| `tests/e2e/primary-flow.spec.ts` | removed | Removed the WebdriverIO/Mocha version of the smoke flow. |
| `tests/e2e/primary-flow.mjs` | added test | Ported the same layout, navigation, IPC-mock, and call-history assertions to Playwright plus strict Node assertions. |
| `wdio.conf.ts` | removed | Removed obsolete WebdriverIO runner configuration. |
| `README.md` | modified | Documented the one-time managed Chromium install and updated the E2E description. |
| `src-tauri/Cargo.toml` | modified | Synchronized the patch version to 0.2.1. |
| `src-tauri/Cargo.lock` | modified | Synchronized the locked application package version to 0.2.1. |
| `src-tauri/tauri.conf.json` | modified | Synchronized the patch version to 0.2.1. |

## Diff Highlights

- The direct browser-test dependency count changed from five `@wdio/*` packages to one `playwright-core` package.
- `npm install` removed 447 installed packages from the previous development graph.
- The smoke flow now launches Chromium, opens the Vite app, runs the existing assertions, and always closes both browser and server.

## Tests Added or Updated

- `tests/e2e/primary-flow.mjs::runPrimaryFlow` — preserves desktop/compact layout checks, sidebar navigation checks, IPC mock setup, and command call-history assertions.

## Local Verification

- Commands run: `npm audit --json` → pass, 0 vulnerabilities.
- Commands run: `npm test` → pass, 10 files and 20 tests.
- Commands run: `npm run test:e2e` → pass, primary desktop flow completed in headless Chromium.
- Commands run: `npm run lint` → pass.
- Commands run: `npm run format:check` → pass.
- Commands run: `npm run build` → pass.
- Commands run: `git diff --check` → pass.
- Manual checks: confirmed the removed graph contained only development dependencies and that the managed Chromium cache is outside the repository.

## Deviations from Assessment

- The host had no stable Google Chrome installation, so the runner uses Playwright-managed Chromium rather than the `chrome` channel. Added `test:e2e:install` and README guidance so this requirement is explicit and reproducible.
- Added `README.md` to the change scope to document the newly required browser-install step.
- The pre-existing uncommitted `package-lock.json` changes removed optional-package `libc` metadata. Regenerating the lockfile with the same npm version retained/subsumed those metadata removals alongside the security changes.

## Follow-ups

- Install the managed browser with `npm run test:e2e:install` on fresh CI and developer hosts before running `npm run test:e2e`.
- Continue with the full bug verification workflow, including Rust checks, Tauri release build, and local application installation.
