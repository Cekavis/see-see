# Implementation Plan: Anthropic Messages Reasoning Effort

**Branch**: `master` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)

## Summary

Extend the existing shared model reasoning setting to Anthropic Messages. The settings UI will use Anthropic's five official effort values and default to `low`; the Rust validation layer will default and validate Anthropic values; the Anthropic adapter will send the value as `output_config.effort`. No schema migration or dependency is needed.

## Technical Context

**Language/Version**: TypeScript 6 / React 19; Rust edition 2024

**Primary Dependencies**: Existing React, Tauri 2, serde, serde_json, reqwest, Vitest, cargo test

**Storage**: Existing SQLite `model_configs.reasoning_effort` column; no schema change

**Testing**: Vitest frontend tests; Rust provider contract and model configuration tests; lint, formatting, frontend build, and cargo test

**Target Platform**: Windows and macOS Tauri desktop app

**Project Type**: Desktop app with React UI and Rust provider adapters

**Performance Goals**: No additional network request or measurable settings-page cost

**Constraints**: Reuse existing enum and controls; preserve OpenAI behavior; keep API keys out of JSON bodies

**Scale/Scope**: One settings field, one protocol adapter, and their focused regressions

## Research

Anthropic's current Messages API documents `output_config.effort` with exactly `low`, `medium`, `high`, `xhigh`, and `max`. The implementation maps the existing `ReasoningEffort` enum to this field and does not add the separate adaptive-thinking parameter, preserving compatibility with existing configured models while enabling the requested official effort control.

Source: [Anthropic Messages API reference](https://platform.claude.com/docs/en/api/messages/create)

## Constitution Check

- **Maintainability**: Reuses existing protocol, enum, field, validation, and request-building patterns; no dependency or abstraction layer is added.
- **Testing**: Adds frontend selector coverage, Rust request-contract coverage, and persistence/default validation.
- **User experience**: Keeps the existing settings field and error flow while showing only protocol-supported values.
- **UI quality**: Reuses `Field` and the existing select control; no new CSS or visual system is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/020-anthropic-messages-reasoning/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── anthropic-reasoning.md
└── tasks.md
```

### Source Code

```text
src/ipc.ts
src/views/Settings.tsx
src/views/Settings.model.test.tsx
src-tauri/src/providers/mod.rs
src-tauri/src/providers/anthropic.rs
src-tauri/src/settings.rs
src-tauri/tests/provider_contracts.rs
src-tauri/tests/model_config.rs
```

**Structure Decision**: Keep the current Tauri split. Frontend protocol-specific option presentation stays in `Settings.tsx`; shared protocol validation remains in the Rust provider/settings modules; request shape remains owned by the Anthropic adapter.

## Complexity Tracking

No constitution violations or new dependencies.
