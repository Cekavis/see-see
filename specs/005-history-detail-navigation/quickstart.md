# Quickstart: Validate History Detail Navigation

## Prerequisites

- Locked dependencies installed with `npm install`.
- At least two saved history records; one result should contain a blank line.

## Automated validation

```sh
npm test -- src/views/History.test.tsx
npm run lint
npm run format:check
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all commands pass, including regressions for dedicated detail navigation, return-state restoration, and summary whitespace.

## Manual UI validation

1. Run `npm run tauri dev` and open History at approximately 1094×768.
2. Confirm the sidebar is narrower and all brand/navigation labels remain visible.
3. Enter values in the result and prompt filters, choose a status, search, load more if available, and scroll partway down.
4. Open a record. Confirm the list and filters are replaced by a detail view with `返回历史记录` at the top.
5. Exercise copy and, where an image exists, resubmit.
6. Return. Confirm all filter values, loaded cards, and the exact scroll position are retained without a visible jump.
7. Confirm summaries preserve line breaks and blank lines and do not overflow horizontally.
8. Repeat list and detail review around 780×800 and 540×800 to verify responsive navigation, spacing, wrapping, and keyboard focus order.

## Release validation

Run `npm run tauri build`, install the produced local application bundle, and repeat the primary open/return flow in the installed app.

## Validation record (2026-07-26)

- Focused History tests: 4 passed, including dedicated detail navigation, detail scroll reset, filter/list/scroll restoration, no return query, load-failure recovery, copy/resubmit, and multiline summary markup.
- Full frontend tests: 40 passed across 12 files.
- ESLint and Prettier checks: passed.
- Frontend production build: passed for version 0.4.0.
- Rust tests: passed after allowing the provider contract tests to bind temporary loopback ports; the initial sandboxed run failed only those two port bindings with `PermissionDenied`.
- Responsive browser review: at 1094×768 the sidebar measured 184 px (16.4% narrower than 220 px) and filters had no overflow; at 780×800 navigation moved to the top and the page had no horizontal overflow; at 540×800 navigation was horizontally scrollable, filters were single-column, and the page had no horizontal overflow.
- Installed-app review at 1024×720: the 184 px sidebar retained all labels; multiline summaries visibly preserved line breaks and blank lines; selecting a record near the bottom replaced the list with a polished detail page whose return action stayed at the top; the screenshot, metadata, result area, copy action, and resubmit action remained usable.
- Installed-app return review: after filtering by prompt and scrolling near the bottom, returning from detail preserved the `日语学习解析` filter value and restored the prior bottom-list scroll position.
- The first sandboxed signing attempt could not see the login keychain and correctly produced no ad-hoc package. The escalated release build found the existing `See See Local Release` identity and successfully produced signed `See See.app` and `See See_0.4.0_aarch64.dmg` bundles.
- `npm run verify:macos-signature` passed with bundle identifier `app.seesee.desktop` and certificate leaf `095d7f5883674a4a3a0d219b60c69b6168c9844f`.
- The signed app was installed to `/Applications/See See.app`; its version is 0.4.0, deep strict signature verification passes, and its designated requirement matches the previously installed 0.3.6 identity.
