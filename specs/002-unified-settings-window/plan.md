# Implementation Plan: See See 统一设置主窗口

**Branch**: `master` | **Date**: 2026-07-23 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-unified-settings-window/spec.md`

## Summary

Replace four management windows with one React-owned settings shell in the existing `main` webview. Reuse current model, prompt, history and desktop-setting components as section content, remove the `open_view` IPC/window lifecycle, route tray capture through the existing capture action, and apply a dependency-free Windows Raycast-inspired CSS system with native controls and dialogs.

## Technical Context

**Language/Version**: TypeScript 6, React 19, Rust 1.95

**Primary Dependencies**: Existing React, Tauri 2 and Tauri plugins only; no new dependencies

**Storage**: Existing SQLite and operating-system credential store; no migration

**Testing**: Vitest/Testing Library, WebdriverIO, cargo test, Clippy, manual desktop visual review

**Target Platform**: Windows 10/11 and macOS 14+

**Project Type**: Tauri desktop application

**Performance Goals**: Section switching is immediate and does not create a webview; existing capture latency targets remain unchanged

**Constraints**: Native titlebars, no transparent windows, no router/state/UI/icon libraries, preserve all business behavior and accessibility

**Scale/Scope**: Five management sections, one main management window, existing capture and result windows

## Constitution Check

- **Maintainability — PASS**: Deletes dynamic management-window infrastructure and uses local React state plus existing components.
- **Testing — PASS**: Adds focused tests for navigation, dialog behavior and tray/window regressions while preserving all existing suites.
- **User experience — PASS**: Defines consistent navigation, loading/error/empty states and predictable desktop lifecycle.
- **UI quality — PASS**: Centralizes tokens, preserves native semantics, supports representative sizes and requires visual review.

## Project Structure

### Documentation

```text
specs/002-unified-settings-window/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/ui-navigation.md
├── checklists/requirements.md
└── tasks.md
```

### Source Code

```text
src/
├── App.tsx
├── ipc.ts
├── styles.css
├── components/
└── views/

src-tauri/src/
├── lib.rs
└── commands.rs

tests/e2e/primary-flow.spec.ts
```

**Structure Decision**: Keep the current single Tauri project. Add only a settings-shell view and two small shared visual primitives; refactor existing views for embedding instead of introducing routing or a component framework.

## Complexity Tracking

No constitution violations or dependency additions.
