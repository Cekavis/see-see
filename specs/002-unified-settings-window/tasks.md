# Tasks: See See 统一设置主窗口

**Input**: Design documents from `/specs/002-unified-settings-window/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ui-navigation.md

## Phase 1: Setup

- [x] T001 Record the approved feature artifacts in specs/002-unified-settings-window/
- [x] T002 Synchronize version 0.2.0 in package.json, src-tauri/Cargo.toml, and src-tauri/tauri.conf.json

## Phase 2: Shared UI Foundation

- [x] T003 [P] Add local icon and accessible confirmation-dialog primitives in src/components/
- [x] T004 [P] Replace global styling with the responsive light/dark design system in src/styles.css
- [x] T005 Refine src-tauri/icons/app-icon.svg and regenerate all derived platform icons

## Phase 3: User Story 1 - Unified Settings Window (P1)

**Goal**: Manage all application sections in one main window.

**Independent Test**: Switch all five sections without creating another management window.

- [x] T006 [P] [US1] Add settings-shell navigation tests in src/views/SettingsShell.test.tsx
- [x] T007 [P] [US1] Update onboarding navigation tests in src/views/Onboarding.test.tsx
- [x] T008 [US1] Implement the unified shell and embedded general/about sections in src/views/SettingsShell.tsx
- [x] T009 [US1] Refactor src/views/Settings.tsx, src/views/Prompts.tsx, and src/views/History.tsx for embedded section layouts
- [x] T010 [US1] Move save-history behavior into src/views/DesktopSettings.tsx and embed onboarding in the general section
- [x] T011 [US1] Simplify src/App.tsx and src/ipc.ts to main/capture/result views with local section navigation

## Phase 4: User Story 2 - Shortcut and Tray Capture (P1)

**Goal**: Keep capture available through the global shortcut and a minimal tray menu.

**Independent Test**: Both triggers enter the same existing capture flow while management entries only focus main.

- [x] T012 [P] [US2] Update desktop lifecycle regression coverage for main-window close behavior
- [x] T013 [US2] Replace the tray menu and remove dynamic management windows in src-tauri/src/lib.rs and src-tauri/src/commands.rs
- [x] T014 [US2] Update tests/e2e/primary-flow.spec.ts for unified navigation and removal of window capture/open_view controls

## Phase 5: User Story 3 - Accessible Visual System (P2)

**Goal**: Deliver the approved Windows Raycast-inspired visual system across all states.

**Independent Test**: Keyboard, themes, dialogs and representative viewport checks pass.

- [x] T015 [P] [US3] Update deletion tests in src/views/Settings.model.test.tsx, src/views/Prompts.test.tsx, and src/views/History.test.tsx
- [x] T016 [US3] Apply shared icons, switches, status treatments and dialogs across src/views/
- [x] T017 [US3] Verify result and capture styling in src/views/Result.tsx and src/views/CaptureOverlay.tsx

## Phase 6: Polish and Validation

- [x] T018 Run frontend formatting, typecheck, lint, unit tests, build, and E2E checks
- [x] T019 Run cargo fmt, Clippy, and Rust tests
- [x] T020 Build and install the Windows bundle, then record validation evidence and manual visual gaps in quickstart.md
- [x] T021 Review git diff, mark tasks complete, commit atomically, and push origin/master

## Dependencies and Execution Order

- T001–T002 establish traceability and release metadata.
- T003–T005 are independent shared foundations.
- US1 and US2 may proceed in parallel after the shared foundation; US3 integrates their UI.
- Validation and delivery require all implementation tasks.

## Parallel Opportunities

- Shared components/styles/icon work can run independently.
- Frontend shell work and Rust tray/window work touch separate files.
- Unit-test updates can run alongside Rust lifecycle tests.

## Implementation Strategy

Implement the shell and window deletion first, then apply the visual system and dialogs, and finish with the complete desktop lifecycle and visual validation matrix.
