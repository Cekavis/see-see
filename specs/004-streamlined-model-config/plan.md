# Implementation Plan: Streamlined Model Configuration

**Branch**: `master` | **Date**: 2026-07-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/004-streamlined-model-config/spec.md`

## Summary

Make model configurations self-contained SQLite records with a plain-text API key, migrate readable legacy system credentials on startup, and remove persisted connection-test state and activation gating. Refactor the model page into a list-first state machine whose editor exists only during add/edit, whose test action uses the draft without saving, and whose cards support immediate copying. Remove generic descriptive text beneath primary page titles across the app while retaining contextual safety, privacy, validation, and state guidance.

## Technical Context

**Language/Version**: TypeScript 6.0, React 19.2, Rust edition 2024, CSS

**Primary Dependencies**: Tauri 2, React, rusqlite 0.37, reqwest 0.12, existing keyring/secrecy compatibility bridge; no new dependency

**Storage**: Local SQLite `model_configs` rows gain a nullable plain-text `api_key`; legacy `credential_ref` is read only for one-way migration; connection-test columns are cleared and no longer read or written

**Testing**: Vitest 4 with Testing Library; Rust unit/integration tests with in-memory SQLite and mock credential stores; existing browser smoke flow; native macOS visual review

**Target Platform**: Tauri 2 desktop application on macOS and Windows

**Project Type**: Desktop application with React frontend and Rust/Tauri backend

**Performance Goals**: Initial list and editor transitions remain immediate for typical local configuration counts; migration performs one credential lookup per legacy configuration only once

**Constraints**: 720×520 minimum window, exact API-key byte-for-byte text preservation, HTTPS validation for remote endpoints, no saved-key exposure to the webview or non-configuration surfaces, legacy keyring failures must not block startup, no new dependency

**Scale/Scope**: Five primary settings sections, model configuration CRUD/copy/activation/test/list flows, local databases created before and after version 0.3.0; expected configuration counts are small (tens, with copy naming safe through 10,000 attempts)

## Constitution Check

*GATE: Passed before Phase 0 and re-checked after Phase 1.*

- **Maintainability**: Pass. The design removes test-state lifecycle branches, reuses direct SQLite settings functions and existing view primitives, and adds only one compatibility migration path. No dependency or new abstraction layer is introduced.
- **Testing**: Pass. Rust regression tests cover storage, migration, copying, activation, and transient tests; frontend tests cover the editor state machine and draft-only testing; existing suites remain enabled.
- **User experience**: Pass. Loading, empty, add, edit, cancel, saving, testing, copying, deleting, disabled, success, error, and recovery states are specified using existing notifications and controls.
- **UI quality**: Pass. Existing tokens, `Button`, `Field`, card, header, notification, and dialog patterns are reused. Keyboard access, 1024×720 and 720×520 layouts, focus visibility, wrapping, and subtitle hierarchy are validation gates.

Post-design re-check: The storage model, IPC/UI contract, and validation guide preserve all four gates. Plain-text key storage is an explicit user requirement and a documented security tradeoff; redaction and non-configuration disclosure boundaries remain mandatory. No other complexity exception is required.

## Project Structure

### Documentation (this feature)

```text
specs/004-streamlined-model-config/
├── checklists/requirements.md
├── contracts/model-configuration.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── ipc.ts
├── styles.css
└── views/
    ├── History.tsx
    ├── Onboarding.tsx
    ├── Prompts.tsx
    ├── Settings.model.test.tsx
    ├── Settings.tsx
    └── SettingsShell.tsx

src-tauri/
├── migrations/
│   └── 0002_plaintext_model_keys.sql
├── src/
│   ├── commands.rs
│   ├── database.rs
│   ├── lib.rs
│   ├── settings.rs
│   └── state.rs
└── tests/
    └── model_config.rs
```

**Structure Decision**: Keep the current Tauri split. Extend the existing database initialization and settings module for schema compatibility and model operations, keep provider connection testing in commands, and refactor only the existing settings views and IPC types. The keyring credential module remains temporarily as a read-only upgrade bridge for previously saved keys; saved keys remain redacted from list summaries and copying occurs in Rust.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Plain-text API key at rest | Explicit user requirement that the key be stored with the endpoint | Keeping OS credential storage contradicts the requested data model; obfuscation would still be reversible while falsely implying protection |
