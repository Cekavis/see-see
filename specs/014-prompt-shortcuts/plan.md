# Implementation Plan: Per-prompt screenshot shortcuts

## Summary
Extend prompt persistence and global shortcut registration to associate each shortcut with a prompt, and redesign prompt management around a dedicated editor view.

## Technical Context
TypeScript/React frontend, Rust/Tauri backend, SQLite storage, Vitest and cargo tests. Reuse existing shortcut parser and global shortcut plugin.

## Constitution Check
Maintainable Quality: reuse existing IPC and settings patterns. Testing: add focused UI and backend regression checks. User experience/UI: preserve shared controls, feedback, accessibility, and visual review.

## Project Structure
`src/views/Prompts.tsx`, `src/ipc.ts`, `src/styles.css`, `src-tauri/src/settings.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, migrations and tests.

## Complexity Tracking
None.
