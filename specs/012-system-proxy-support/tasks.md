# Tasks: System Proxy Support

**Input**: Design documents from `/specs/012-system-proxy-support/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/network-proxy.md, quickstart.md

**Tests**: Configuration assertions are written before each dependency change so they fail without the required proxy feature.

## Phase 1: Setup

**Purpose**: Confirm the existing network paths and release baseline.

- [x] T001 Confirm the shared model client, updater client construction, clean `master` branch, and current v0.10.2 release baseline in `src-tauri/src/providers/mod.rs`, `src-tauri/src/lib.rs`, and repository Git metadata

---

## Phase 2: User Story 1 - System proxy for model requests (Priority: P1) 🎯 MVP

**Goal**: Model listing, connection tests, and analysis streaming discover supported operating-system proxy settings.

**Independent Test**: The release configuration check requires `system-proxy` on reqwest 0.12, and Cargo's resolved feature tree contains the feature.

### Tests for User Story 1

- [x] T002 [US1] Add a failing model-client system proxy dependency assertion in `scripts/verify-release-config.mjs`

### Implementation for User Story 1

- [x] T003 [US1] Enable reqwest 0.12 `system-proxy` for the shared model client in `src-tauri/Cargo.toml` and refresh `src-tauri/Cargo.lock`
- [x] T004 [US1] Run the release configuration assertion and inspect the reqwest 0.12 resolved feature tree from `src-tauri/Cargo.toml`

**Checkpoint**: The shared model client has operating-system proxy discovery without application-level proxy code.

---

## Phase 3: User Story 2 - System proxy for updater requests (Priority: P2)

**Goal**: Update metadata checks and package downloads discover supported operating-system proxy settings.

**Independent Test**: The release configuration check requires the updater reqwest alias, and Cargo's resolved reqwest 0.13 feature tree contains `system-proxy`.

### Tests for User Story 2

- [x] T005 [US2] Add a failing updater system proxy dependency assertion in `scripts/verify-release-config.mjs`

### Implementation for User Story 2

- [x] T006 [US2] Add the minimal reqwest 0.13 feature-unification alias for the updater in `src-tauri/Cargo.toml` and refresh `src-tauri/Cargo.lock`
- [x] T007 [US2] Run the release configuration assertion and inspect the reqwest 0.13 resolved feature tree from `src-tauri/Cargo.toml`

**Checkpoint**: Both model and updater network paths have operating-system proxy discovery.

---

## Phase 4: Release and cross-cutting validation

**Purpose**: Synchronize the compatible feature version, validate, install, and publish.

- [x] T008 Synchronize version 0.11.0 in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`
- [x] T009 Run formatting, lint, frontend tests/build, Rust tests, release configuration checks, and dependency feature verification across the repository
- [x] T010 Produce signed Windows release bundles with `npm run tauri build` and install the generated installer locally
- [x] T011 Inspect final status and diff, commit the complete change atomically, push `master`, create annotated tag `v0.11.0`, and push the tag
- [x] T012 Monitor `.github/workflows/release.yml` until the GitHub Release is published with all expected Windows and macOS assets

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **User Story 1 (Phase 2)**: Depends on Setup and is the MVP.
- **User Story 2 (Phase 3)**: Depends on Setup; may follow User Story 1 because both update `src-tauri/Cargo.toml` and the same regression script.
- **Release (Phase 4)**: Depends on both user stories.

### User Story Dependencies

- **User Story 1 (P1)**: Independent behavior through the shared model client.
- **User Story 2 (P2)**: Independent behavior through the updater client, implemented sequentially to avoid same-file conflicts.

### Parallel Opportunities

- No implementation tasks should run in parallel because both stories modify `src-tauri/Cargo.toml` and `scripts/verify-release-config.mjs`.
- Independent validation commands in T009 may run concurrently when they do not contend for build outputs.

## Implementation Strategy

1. Add the model dependency assertion, confirm failure, then enable its existing dependency feature.
2. Add the updater dependency assertion, confirm failure, then enable the transitive dependency feature through Cargo unification.
3. Synchronize version 0.11.0 and run the full release gate.
4. Install the signed Windows bundle, publish `master` and `v0.11.0`, then monitor the release workflow.
