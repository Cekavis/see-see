# Implementation Plan: Thinking Stream Display

**Branch**: `master` | **Date**: 2026-08-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/007-thinking-stream-display/spec.md`

## Summary

Extend the existing provider stream normalization with a thinking delta, classify the structured fields already returned by supported protocols, and split a leading `<think>` section in the shared stream path so chunk boundaries do not leak tags into the answer. Carry thinking alongside answer text in the existing analysis snapshot, persist it in a separate backward-compatible history column, and render it with a native disclosure that is open during thinking-only streaming and collapsed for answers and history detail.

## Technical Context

**Language/Version**: Rust 1.95, TypeScript 6

**Primary Dependencies**: Tauri 2, React 19, reqwest/eventsource-stream, serde_json; no new dependency

**Storage**: Existing SQLite history gains one nullable thinking text column through a backward-compatible migration

**Testing**: Rust contract and analysis-flow tests with `cargo test`; React behavior tests with Vitest and Testing Library

**Target Platform**: Windows and macOS desktop

**Project Type**: Tauri desktop application

**Performance Goals**: Classify each stream chunk synchronously with bounded tag-prefix buffering and no perceptible UI delay

**Constraints**: Preserve answer-only output, previews, search, and copy; store thinking separately; keep provider compatibility; add no dependency

**Scale/Scope**: Three existing provider protocols, one active analysis, one result view

## Constitution Check

*GATE: Passed before research and after design.*

- **Maintainability**: Extends the existing provider event and analysis snapshot rather than adding a parallel pipeline; one shared tag parser handles all OpenAI-compatible tag streams.
- **Testing**: Adds focused provider fixtures, tag-boundary regression coverage, analysis accumulation checks, and disclosure transition tests.
- **User experience**: Reuses current lifecycle status, error, cancel, retry, copy, and window controls; only thinking presentation is added.
- **UI quality**: Uses native `<details>/<summary>` keyboard behavior and existing design tokens; desktop and compact visual review are required.

## Project Structure

### Documentation (this feature)

```text
specs/007-thinking-stream-display/
├── checklists/requirements.md
├── contracts/thinking-stream.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
src-tauri/src/providers/mod.rs
src-tauri/src/providers/openai.rs
src-tauri/src/providers/anthropic.rs
src-tauri/src/providers/gemini.rs
src-tauri/src/analysis.rs
src-tauri/src/database.rs
src-tauri/src/history.rs
src-tauri/migrations/0003_history_thinking.sql
src-tauri/tests/provider_contracts.rs
src-tauri/tests/analysis_flow.rs
src-tauri/tests/history_integration.rs
src/ipc.ts
src/App.tsx
src/App.test.ts
src/views/Result.tsx
src/views/Result.test.tsx
src/views/History.tsx
src/views/History.test.tsx
src/styles.css
package.json
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
```

**Structure Decision**: Keep protocol-specific field extraction in the existing adapters, cross-chunk `<think>` handling in the shared provider stream, lifecycle accumulation in the existing analysis model, persistence in the existing history module, and presentation in the existing result and history views. Reuse one disclosure component; no dependency is needed.

## Complexity Tracking

No constitution violations or additional complexity are required.
