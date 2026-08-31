# Implementation Plan: Concurrent Analysis Requests

**Branch**: `master` | **Date**: 2026-08-31 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/013-concurrent-analysis-requests/spec.md`

## Summary

Replace the singleton runtime analysis slot with a run-ID keyed collection. Keep each result window and event stream bound to its run ID, and store the complete request input on the run so retries never read a newer active configuration.

## Technical Context

**Language/Version**: TypeScript/React 19; Rust 2021 with Tauri 2

**Primary Dependencies**: Tauri Channels, Tokio, reqwest, Vitest, cargo test

**Storage**: Existing in-memory runtime state and SQLite history database

**Testing**: Vitest and cargo test

**Target Platform**: Windows and macOS desktop

**Project Type**: Tauri desktop application

**Performance Goals**: Starting a second run must not wait for the first run to complete; per-run event dispatch remains O(1) lookup by run ID.

**Constraints**: Preserve capture singleton behavior, existing result-window labels, cancellation semantics, history behavior, and no new dependencies.

**Scale/Scope**: Multiple simultaneous model requests bounded by provider/system resources; no fixed application-level singleton.

## Constitution Check

- **Maintainability**: Uses a standard `HashMap<String, Arc<ActiveAnalysis>>` and existing run-ID window labels; no new abstraction or dependency.
- **Testing**: Adds focused Rust tests for independent runs and original retry snapshots, plus existing frontend reducer and wiring safeguards.
- **User experience**: Keeps existing per-window controls and terminal states; actions remain run-specific.
- **UI quality**: No new visual pattern; only protects result-window identity during attach.

## Project Structure

```text
src/App.tsx                         # attach snapshot identity/ordering guard
src-tauri/src/state.rs              # run-ID keyed runtime collection
src-tauri/src/analysis.rs           # request snapshot and retry input
src-tauri/src/commands.rs           # concurrent lifecycle and run lookup
src-tauri/src/lib.rs                # per-window close and app-exit cleanup
src-tauri/tests/analysis_flow.rs    # concurrency and retry regression tests
src-tauri/tests/desktop_lifecycle.rs# command/lifecycle contract tests
```

**Structure Decision**: Keep the existing single-project Tauri layout and change only runtime analysis ownership and its tests.

## Complexity Tracking

No constitution violations. The map and request snapshot are the smallest structures that support independent windows and deterministic retries.
