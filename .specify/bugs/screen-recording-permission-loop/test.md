# Bug Verification: Screen recording permission prompt repeats on every launch

- **Slug**: screen-recording-permission-loop
- **Tested**: 2026-07-25
- **Assessment**: ./assessment.md
- **Fix**: ./fix.md
- **Result**: partial

## Summary

The installed 0.2.3 app launched, quit, and relaunched without reproducing the unsolicited macOS Screen Recording dialog, and all automated permission-chain and regression checks passed. Verification is marked partial because the freshly built ad-hoc signed binary has a new macOS TCC identity and the explicit OS permission grant was not accepted during automated verification, so persistence after a new grant remains a user-driven manual check.

## Checks Performed

| Check | Command / Action | Result | Notes |
| --- | --- | --- | --- |
| Reproduction (post-fix) | Install `/Applications/See See.app`, launch, quit fully, and launch again | pass | Neither launch showed `universalAccessAuthWarn` or a Screen Recording dialog; the passive UI showed “尚未请求屏幕录制权限”. |
| Explicit OS grant and return from Settings | User confirmation/interaction with macOS Privacy & Security | not-run | Skipped to avoid changing a security-sensitive OS permission without action-time user confirmation; automated UI coverage validates request/denial/focus transitions. |
| New / updated frontend tests | `npm test -- --run src/views/Onboarding.test.tsx` | pass | 6/6 tests passed. |
| Frontend regression suite | `npm test` | pass | 12 files, 29 tests passed. |
| Rust regression suite | `cargo test --manifest-path src-tauri/Cargo.toml` | pass | 36 tests passed with loopback access enabled for wiremock. |
| Browser smoke flow | `npm run test:e2e` | pass | Primary desktop flow passed with local Vite port access enabled. |
| Lint / type-check / format | `npm run lint`; `npm run typecheck`; `npm run format:check`; `git diff --check` | pass | No lint, type, formatting, or whitespace errors. |
| Frontend production build | `npm run build` | pass | TypeScript and Vite production build completed. |
| Desktop release build | `npm run tauri build` | pass | Built `See See.app` and `See See_0.2.3_aarch64.dmg`. |
| Local installation | `ditto … /Applications/See See.app` plus Info.plist readback | pass | Installed version and bundle version are `0.2.3`; bundle identifier is `app.seesee.desktop`. |

## Output Excerpts

```text
Test Files  12 passed (12)
Tests       29 passed (29)

Rust: 36 passed; 0 failed
✓ See See primary desktop flow
All matched files use Prettier code style!

Finished 2 bundles at:
  src-tauri/target/release/bundle/macos/See See.app
  src-tauri/target/release/bundle/dmg/See See_0.2.3_aarch64.dmg
```

Installed UI readback on both launches:

```text
Window: "See See", App: See See
屏幕截图权限
尚未请求屏幕录制权限
请求屏幕录制权限
Matching running apps: See See only
```

## Residual Risks

- The locally built app is ad-hoc signed, so installing a new build can change the TCC identity and require a fresh one-time Screen Recording grant even when the bundle identifier and path are unchanged.
- The explicit macOS grant, denial, System Settings return, and post-grant relaunch sequence is covered by Rust/React tests but was not completed manually because it changes a security-sensitive OS permission.
- Windows permission behavior remains on xcap's existing path and still requires the repository's documented Windows manual check for capture changes.

## Recommendation

The code fix and release artifacts are ready: the original unsolicited startup prompt is gone in the installed app and automated regressions are clean. After the user clicks “请求屏幕录制权限” and grants the newly signed 0.2.3 build once, perform one final quit/relaunch check; if the UI reports “屏幕权限已就绪” and no dialog appears, this verification can be promoted from partial to fully verified.
