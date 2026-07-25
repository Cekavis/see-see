# Bug Assessment: Screen recording permission prompt repeats on every launch

- **Slug**: screen-recording-permission-loop
- **Created**: 2026-07-25
- **Source**: pasted text and app screenshot
- **Verdict**: valid
- **Severity**: high

## Report (verbatim or summarized)

> 我现在每次打开 see see 都会弹出这个，即使我已经授权了，请你修复整个授权请求和展示链路存在的问题

The attached macOS dialog says that “See See” would like to record the computer's screen and audio and asks the user to open Privacy & Security settings or deny the request. It appears whenever the app opens even though Screen Recording access was already granted.

## Symptom

Launching See See performs a real screen capture while loading the application snapshot, so macOS can display a system recording authorization dialog during startup. Permission checking, requesting, opening Settings, and reflecting a newly granted status are not separated, which causes an intrusive repeated prompt and stale onboarding UI.

## Reproduction

1. Grant See See Screen Recording access in macOS Privacy & Security.
2. Quit and reopen See See.
3. Observe the system Screen Recording authorization dialog during launch instead of a passive permission check.
4. When permission is not granted, click the onboarding recovery control and observe that it opens System Settings but refreshes the status immediately, before the user can change the setting.

## Suspected Code Paths

- `src-tauri/src/capture.rs:screen_permission_status()` — enumerates a monitor and captures a 1×1 region to infer permission; this is an active screen-recording operation, not a passive authorization query.
- `src-tauri/src/settings.rs:load_app_snapshot()` — invokes the active probe whenever any app snapshot is loaded, including application startup and multiple settings operations.
- `src-tauri/src/lib.rs:run()` — loads an app snapshot in Tauri setup, making the active probe part of every launch.
- `src-tauri/src/commands.rs:begin_capture_action()` — starts full capture without a passive permission guard, allowing capture APIs to become an implicit permission-request path.
- `src-tauri/src/commands.rs:open_screen_permission_settings()` — only opens Settings; there is no explicit command for the initial system permission request.
- `src/views/Onboarding.tsx` — maps both unknown and denied states to the same recovery path and refreshes immediately after opening Settings, so the displayed state can remain stale after the user grants access.

## Root Cause Hypothesis

Confidence: high. `screen_permission_status()` calls `Monitor::all()` and then `capture_region(0, 0, 1, 1)`. On macOS, xcap implements this through `CGWindowListCreateImage`, which is a protected screen-capture operation and can trigger TCC authorization UI. Because `load_app_snapshot()` calls this function and is used during startup, a permission-sensitive operation happens on every launch. The flow also lacks an explicit `CGRequestScreenCaptureAccess` action and a post-Settings focus refresh, so authorization request and display state are coupled incorrectly.

## Proposed Remediation

**Preferred**: On macOS, replace the capture-based status probe with CoreGraphics `CGPreflightScreenCaptureAccess`, which reads the current grant without initiating capture or requesting access. Add a dedicated explicit request function backed by `CGRequestScreenCaptureAccess`, expose it through IPC, and guard actual capture with the passive preflight so tray and shortcut actions cannot become implicit authorization prompts.

Update onboarding to distinguish the unconfirmed/not-granted state from a request that returned denied. Offer an explicit “request permission” action first, show the System Settings recovery only after denial, and re-run the passive status check when the app window regains focus after the user returns from Settings. Keep non-macOS behavior compatible with the existing capture backend.

**Files likely to change**:

- `src-tauri/Cargo.toml`
- `src-tauri/src/capture.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`
- `src/ipc.ts`
- `src/views/Onboarding.tsx`
- `src/views/Onboarding.test.tsx`
- `src-tauri/tests/capture_flow.rs`
- `package.json`
- `src-tauri/tauri.conf.json`

**Tests to add or update**:

- Verify passive macOS preflight mapping reports granted or unknown without using the capture backend.
- Verify denied capture attempts return the screen-permission error before monitor capture begins.
- Verify onboarding explicitly requests permission, displays denial recovery, refreshes on focus, and completes once the passive status becomes granted.

## Risks & Considerations

- CoreGraphics cannot reliably distinguish “not requested yet” from “previously denied” using preflight alone; the UI should call the explicit request action and use its result to select the recovery message for the current session.
- macOS permission grants are associated with the signed application identity. Local installation must use the same bundle identifier and stable installed app path for manual verification.
- The authorization dialog itself cannot be exercised safely in automated tests; verification must combine unit/UI coverage with a locally installed macOS manual launch check.
- The behavioral fix requires a synchronized patch-version increment and a release build/local installation under repository policy.

## Open Questions

- None.
