# Implementation Plan: Result Window Image Preview

**Branch**: `master` | **Date**: 2026-09-16 | **Spec**: [spec.md](./spec.md)

## Summary

Expose the original image held by the active analysis to the matching result window, render it as a bounded comparison preview above the existing result content, and increase the native result-window default and minimum heights by 50%. The implementation reuses the existing Tauri `Response` byte transport, the current-run lookup, the frontend object-URL cleanup pattern used by history, and the existing result layout tokens.

## Technical Context

**Language/Version**: TypeScript 6.0, React 19.2, Rust edition 2024

**Primary Dependencies**: Tauri 2.11, `@tauri-apps/api`, Vitest, Testing Library, Cargo test

**Storage**: In-memory `ActiveAnalysis.image_png`; no schema or migration changes

**Testing**: Vitest frontend tests, focused Rust lifecycle/command tests, TypeScript typecheck, lint, formatting check, and Rust test suite

**Target Platform**: Tauri desktop on Windows and macOS, with the existing responsive result-window CSS

**Project Type**: Desktop application with React frontend and Rust/Tauri backend

**Performance Goals**: Load one already-captured image per result window without disk or network I/O; keep preview retrieval asynchronous so streaming result updates remain responsive

**Constraints**: Preserve current-run isolation, existing result actions and state transitions, no new dependency, no history/database change, no horizontal overflow at 420-pixel minimum width

**Scale/Scope**: One result-window IPC command, one frontend preview row, one shared CSS layout adjustment, updated focused tests and validation artifacts

## Constitution Check

- **Maintainability — PASS**: Reuses `ActiveAnalysis`, the existing `active_analysis` lookup, Tauri `Response`, history-style object URL cleanup, and current result CSS tokens. No new dependency or abstraction is required.
- **Testing — PASS**: Adds frontend regression coverage for image rendering and layout rows, backend/source-contract coverage for the new command registration and current-run lookup, and updates native size assertions.
- **User experience — PASS**: The preview is non-blocking, uses the existing notification pattern on retrieval failure, preserves all result states/actions, and remains bounded at compact widths.
- **UI quality — PASS**: Uses the existing surface, border, radius, and spacing tokens; preserves source proportions; includes default/minimum viewport review criteria for wide/tall images and light/dark appearances.

## Project Structure

### Documentation (this feature)

```text
specs/016-result-window-image-preview/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ipc.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code

```text
src/
├── App.tsx                 # Result-window image loading and IPC wiring
├── ipc.ts                  # Frontend command contract
├── styles.css              # Result preview/layout rules
└── views/
    ├── Result.tsx          # Preview row markup and props
    └── Result.test.tsx     # Component and layout regressions

src-tauri/src/
├── commands.rs             # Current-run image command
├── lib.rs                  # Tauri command registration
└── windowing.rs            # Result default/minimum sizes

src-tauri/tests/
└── desktop_lifecycle.rs    # Native result-window size regression
```

**Structure Decision**: Keep the change in the existing frontend/backend layers. The backend owns run isolation and byte access, `App.tsx` owns the run-scoped object URL lifecycle, `Result.tsx` owns accessible presentation, and `windowing.rs` owns native size constants.

## Implementation Sequence

1. Add the run-scoped image IPC contract and register it in Tauri, returning the original bytes from the matching `ActiveAnalysis` only.
2. Add the frontend IPC wrapper and load/revoke an object URL for the result window's current `runId`; pass the URL to `Result` without coupling image loading to analysis snapshot updates.
3. Render the preview above the result content with meaningful alternative text and add bounded, proportion-preserving CSS. Expand the result grid to account for the optional preview while keeping the result text as the scrolling region.
4. Change result-window default/minimum heights to 750/540 while preserving the existing widths, focus, placement, always-on-top, close, and navigation behavior.
5. Add/update focused tests for image display, retrieval failure tolerance, layout row ownership, command registration/current-run isolation, and native size expectations.
6. Run the feature quickstart, frontend checks, Rust tests, and manual visual review at 460×750 and 420×540 with wide and tall images.

## Risks and Mitigations

- **Image retrieval races with closing a run**: Use the existing active-run lookup and tolerate a failed fetch in the frontend; revoke object URLs during cleanup.
- **Preview consumes too much vertical space**: Cap the image height and retain a `minmax(0, 1fr)` result row so long text remains scrollable.
- **Current-run mix-up**: Require `runId` in the command and read only the matching `ActiveAnalysis`; add a source-contract regression for the lookup.
- **Compact-window clipping**: Test the updated minimum size and retain width auto/max-width behavior without a fixed aspect ratio.

## Complexity Tracking

No constitution violations. No new dependency, storage layer, or public external API is introduced.
