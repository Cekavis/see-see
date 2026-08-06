# Implementation Plan: App Updater

**Branch**: `master` | **Date**: 2026-08-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/008-app-updater/spec.md`

## Summary

Add Tauri's signed updater and restart plugins, point them at the latest GitHub Release metadata, and place the manual check/install flow in the existing About page. Extend the tag-triggered release workflow to sign updater bundles, upload `latest.json`, validate all installer and updater targets while the release is a draft, and publish only after the asset set is complete.

## Technical Context

**Language/Version**: Rust 1.95, TypeScript 6, Node 24+

**Primary Dependencies**: Tauri 2 updater and process plugins, React 19, existing GitHub Actions and `tauri-apps/tauri-action`

**Storage**: No application persistence; update state is in-memory, signing secrets stay outside source control

**Testing**: Vitest and Testing Library for About-page state transitions; Cargo tests and production builds for plugin wiring; workflow format/static assertions and release asset validation

**Target Platform**: Windows x64, macOS Apple Silicon, macOS Intel

**Project Type**: Tauri desktop application

**Performance Goals**: Return a check result within 10 seconds on a stable connection and update progress without blocking the settings UI

**Constraints**: Signed updates only; retain stable macOS application identity; no automatic background installation; incomplete releases remain drafts; no signing material in Git

**Scale/Scope**: One About-page flow, three updater targets, four user-facing installers, one GitHub Release workflow

## Constitution Check

*GATE: Passed before research and after design.*

- **Maintainability**: Uses the official updater/process plugins and the existing About page, Button, notifications, settings rows, and release workflow; no custom updater service or IPC layer.
- **Testing**: Adds focused UI state tests, configuration assertions, existing full test/build checks, and release-time asset validation.
- **User experience**: Defines current, checking, available, installing, restarting, and recoverable failure states with duplicate actions disabled.
- **UI quality**: Reuses existing tokens and responsive rows; keyboard, busy-state, desktop, and compact visual checks are included.

## Project Structure

### Documentation (this feature)

```text
specs/008-app-updater/
├── checklists/requirements.md
├── contracts/update-flow.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
.github/workflows/release.yml
src/views/SettingsShell.tsx
src/views/SettingsShell.test.tsx
src/styles.css
src-tauri/src/lib.rs
src-tauri/capabilities/default.json
src-tauri/tauri.conf.json
package.json
package-lock.json
src-tauri/Cargo.toml
src-tauri/Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the update UI in the existing About view and call the official JavaScript plugin bindings directly. Keep release assembly in the single existing release workflow. Add no custom backend command, service abstraction, storage, or new screen.

## Complexity Tracking

No constitution violations are required. Two official plugins are necessary because update verification/install and application restart are separate platform capabilities.
