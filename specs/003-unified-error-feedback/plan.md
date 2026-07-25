# Implementation Plan: Unified App Feedback

**Branch**: `master` | **Date**: 2026-07-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-unified-error-feedback/spec.md`

## Summary

Replace page-positioned operation messages with one window-scoped notification provider rendered above the document flow. Views publish error or success outcomes through a shared hook; the provider owns ordering, persistence, timeout, accessible announcement, dismissal, optional recovery actions, and compact-window layout. Field validation and persistent failed-history content remain inline.

## Technical Context

**Language/Version**: TypeScript 6.0, React 19.2, CSS; Rust 2021 remains unchanged

**Primary Dependencies**: React, React DOM portal API, Tauri 2 APIs already in the application; no new dependency

**Storage**: In-memory notification state scoped to each webview window; no persistence

**Testing**: Vitest 4, Testing Library, existing frontend view tests, native macOS UI inspection

**Target Platform**: Tauri 2 desktop application on macOS and Windows

**Project Type**: Desktop application with React frontend and Rust/Tauri backend

**Performance Goals**: Feedback rendered in the current viewport within one second; no document reflow; small bounded in-memory queue

**Constraints**: 320×240 minimum representative viewport, light/dark themes, reduced motion, no credential or raw provider-response exposure, independent notification state per window

**Scale/Scope**: Main settings view, result view, and all existing asynchronous action feedback; no backend protocol or persistence change

## Constitution Check

*GATE: Passed before Phase 0 and re-checked after Phase 1.*

- **Maintainability**: Pass. A provider and hook centralize behavior with existing React and CSS primitives. No dependency or backend abstraction is added.
- **Testing**: Pass. Focused component tests cover lifecycle and accessibility; existing view tests are updated for integration, and native compact-window review covers layout behavior.
- **User experience**: Pass. Loading, disabled, empty, field-validation, success, error, recovery, and persistent failure-content states are explicitly separated.
- **UI quality**: Pass. Existing design tokens, `Button`, and `Icon` primitives are reused. Compact viewports, keyboard focus, screen-reader roles, themes, and reduced motion are acceptance gates.

Post-design re-check: The notification contract, state model, and validation guide preserve all four gates. No complexity exception is required.

## Project Structure

### Documentation (this feature)

```text
specs/003-unified-error-feedback/
├── checklists/requirements.md
├── contracts/notification-ui.md
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
├── components/
│   ├── Notifications.tsx
│   ├── Notifications.test.tsx
│   ├── ErrorNotice.tsx
│   └── Icon.tsx
├── views/
│   ├── DesktopSettings.tsx
│   ├── History.tsx
│   ├── Onboarding.tsx
│   ├── Prompts.tsx
│   ├── Result.tsx
│   └── Settings.tsx
├── main.tsx
└── styles.css
```

**Structure Decision**: Add one shared frontend component beside existing design primitives, wrap each React root once in `main.tsx`, and migrate operation feedback in existing views. Keep `ErrorNotice` only for persistent inline failure content or replace it with an explicitly named inline variant if that improves clarity.

## Complexity Tracking

No constitution violations require justification.
