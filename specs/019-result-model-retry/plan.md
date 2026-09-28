# Implementation Plan: Result Model Retry

**Branch**: `master` | **Date**: 2026-09-28 | **Spec**: [spec.md](spec.md)

## Summary

Add a compact scrollable model chooser to the result footer. A new asynchronous IPC command clones the retained request, substitutes the selected model and credentials, and uses the existing independent-window launch path.

## Technical Context

- Languages: TypeScript 6 / React 19; Rust with existing Tauri 2 toolchain.
- Dependencies: Existing React, Tauri, Testing Library, Vitest, and Rust libraries only.
- Storage: Existing configuration and history storage; no schema change.
- Testing: Focused UI/IPC/Rust regression tests, full repository gates, browser visual review, local installer verification.
- Target: Windows and macOS desktop; Windows local verification available.
- Performance: Fixed-height scrollable list supporting 500 configurations without extra network lookups.
- Scope: Result footer, new chooser, one IPC command, retained-input helper, tests, and version synchronization.

## Constitution Check

- Maintainability: Existing `start_analysis`, `ActiveAnalysis`, model loading, Button, and dialog styles reused.
- Testing: Cover new-run independence, immutable source input, selected credentials, loading races, keyboard interaction, failures, double-submit guard, asynchronous native dispatch, and StrictMode subscription cleanup.
- UX: Name-only rows with no search/count/progress copy; empty/error feedback; immediate selection; original result remains visible.
- UI: Existing tokens and dialog surface; review 460×500 and 420×360. Human review remains the user's acceptance step.
- Post-design: No new dependency, persistent setting, or schema; no exceptions needed.

## Project Structure

- `src/components/ModelRetryDialog.tsx` and adjacent tests: scrollable modal, async lifecycle, keyboard controls.
- `src/styles.css`: bounded modal/list and existing-style model rows.
- `src/views/Result.tsx`, `src/App.tsx`, `src/ipc.ts`: footer and backend wiring.
- `src-tauri/src/analysis.rs`, `commands.rs`, `lib.rs`: immutable input substitution and new command.
- `src-tauri/tests/analysis_flow.rs`: input/state isolation regression tests.
- `src-tauri/src/windowing.rs` and `src/App.test.ts`: route native Escape through the webview so an open chooser dismisses before the result window closes; preserve window-close shortcuts otherwise.
- `specs/019-result-model-retry/`: requirements, contracts, tasks, validation evidence.

## Execution and Ownership

After main-agent inspection, backend and standalone chooser are independent implementation tracks. Main agent owns result/IPC wiring, integration tests, versioning, validation, installation, and delivery. Agents do not commit. Optional intermediate commit hooks are deferred to one atomic final commit per repository instruction; branch-creation hook is overridden by the instruction to work on master.

## Complexity Tracking

No new dependencies. A new dialog component is needed because the existing confirmation dialog exposes fixed confirmation actions rather than an immediate-selection list. The result subscription invalidates old event/attach callbacks on effect cleanup; the retry command follows the existing asynchronous window-creation constraint.
