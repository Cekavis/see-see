# Implementation Plan: System Proxy Support

**Branch**: `master` | **Date**: 2026-08-23 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/012-system-proxy-support/spec.md`

## Summary

Enable the networking libraries' existing operating-system proxy discovery for both the shared model client and the updater client. Keep current request construction, TLS, redirect, timeout, credential, and signature-verification behavior unchanged. Record the dependency-feature contract in the existing release configuration check and publish the compatible feature as version 0.11.0.

## Technical Context

**Language/Version**: Rust 2024 edition; TypeScript 6 for release validation

**Primary Dependencies**: reqwest 0.12 for model requests, reqwest 0.13 through tauri-plugin-updater 2.10.1, Tauri 2

**Storage**: N/A

**Testing**: Node assertion-based configuration check, Cargo feature-tree inspection, cargo test, Vitest, ESLint, Prettier, frontend build, signed Tauri release build

**Target Platform**: Windows x64; macOS Apple Silicon and Intel

**Project Type**: Tauri desktop application

**Performance Goals**: Proxy discovery adds no user-visible interaction and preserves existing request timeout limits

**Constraints**: Reuse dependency-native system proxy discovery; no proxy settings UI, new proxy service, redirect relaxation, TLS relaxation, or signature-verification change

**Scale/Scope**: One shared model HTTP client and the updater metadata/download clients

## Constitution Check

*GATE: Passed before Phase 0 and re-checked after Phase 1.*

- **Maintainability**: Pass. Enables an existing dependency feature instead of adding application proxy logic. The updater requires one direct dependency alias solely to unify the transitive reqwest 0.13 feature set.
- **Testing**: Pass. Extends the existing release configuration assertion to fail when either network path loses system proxy support; full existing validation remains enabled.
- **User experience**: Pass. No new UI or interaction states; existing network errors and recovery behavior remain unchanged.
- **UI quality**: Pass. No visual surface changes.

## Project Structure

### Documentation (this feature)

```text
specs/012-system-proxy-support/
├── checklists/requirements.md
├── contracts/network-proxy.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
scripts/
└── verify-release-config.mjs

src-tauri/
├── Cargo.toml
├── Cargo.lock
└── src/providers/mod.rs

package.json
package-lock.json
src-tauri/tauri.conf.json
```

**Structure Decision**: Keep the implementation in dependency configuration because both network paths already use reqwest builders whose default behavior is controlled by enabled features. Reuse the existing release configuration verifier for regression coverage.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Direct alias for reqwest 0.13 | Enables `system-proxy` on the updater plugin's transitive reqwest version without changing updater internals | Upgrading the model client to reqwest 0.13 would also unify TLS backend features and enlarge the regression surface; the plugin does not expose a system-proxy feature flag |
