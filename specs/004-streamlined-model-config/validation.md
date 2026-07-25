# Validation: Streamlined Model Configuration

**Date**: 2026-07-25

**Version**: 0.3.0

**Host**: macOS arm64

## Automated checks

| Check | Result |
| --- | --- |
| `npm run format:check` | Passed; all matched files use Prettier formatting. |
| `npm run lint` | Passed. |
| `npm test` | Passed; 12 files and 34 tests. |
| `npm run build` | Passed; TypeScript and Vite production build completed. |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Passed outside the sandbox; 38 tests. A later sandbox rerun reached the provider contracts with all preceding tests passing, then two WireMock cases were prevented from binding local ports by `EPERM`. |
| `npm run test:e2e` | Passed after the conditional editor assertions were added. The compact viewport was then tightened from 720×720 to 720×520; direct browser review at 720×520 passed, but the automated rerun could not bind `127.0.0.1:1420` because the execution approval service returned 503. |
| `git diff --check` | Passed. |

The focused storage regressions cover exact same-row key values, key preservation and explicit clearing, redacted summaries, untested activation, safe duplication, idempotent legacy migration, and unavailable keyring recovery. Frontend regressions cover closed/add/edit form states, draft-only testing, save without testing, failed-save recovery, copying, ungated activation, busy controls, and delete behavior.

## Release build and installation

`npm run tauri build` completed successfully and produced:

- `src-tauri/target/release/bundle/macos/See See.app`
- `src-tauri/target/release/bundle/dmg/See See_0.3.0_aarch64.dmg`

The bundle was installed to `/Applications/See See.app` with `ditto` and restarted so the installed 0.3.0 build, rather than the previous process, was reviewed.

## UI review

At 1024×720 in the installed macOS app, the model page opened as a saved-configuration list with no editor fields. The “新增配置” action was visible, cards showed endpoint and key-presence only, each card exposed edit/copy/activate/delete actions, and no persisted test status appeared. Opening add mode rendered the editor below the list with the local plain-text key warning and the transient test/cost warning; cancel returned to the list.

At exactly 720×520 in the browser review, `scrollWidth` and `clientWidth` were both 720, no horizontal overflow was present, and the closed model state contained zero editor fields. The conditional editor behavior is additionally covered by the component regression suite and the browser smoke flow.

General, Models, Prompts, History, About, and Onboarding were reviewed for redundant page-title subtitles. Generic subtitles were removed while field hints, privacy information, empty-state guidance, permission recovery, and dynamic readiness text remained contextual.

## Security and compatibility review

- API keys are intentionally stored as plain text in `model_configs.api_key`, in the same SQLite row as the endpoint.
- Saved keys remain outside serialized summaries, accessibility labels, notifications, logs, history, and copy IPC payloads; only `hasApiKey` is exposed.
- The app-data directory and database receive mode `0700` and `0600` respectively on Unix after creation.
- Legacy `credential_ref` and test columns remain schema-only compatibility fields. Startup migration copies available legacy keys into SQLite, deletes the keyring entry, then clears the reference; failures retain the reference for a later retry without blocking startup.
- Testing performs no configuration write and neither saving nor activation requires a test.

## Platform coverage

macOS build, installation, and manual review are complete. Windows packaging and manual validation remain pending because no Windows host is available in this environment.
