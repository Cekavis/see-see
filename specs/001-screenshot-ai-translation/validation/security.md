# Security validation

Date: 2026-07-23

## Result

Source and configuration review passed for the v1 trust boundaries below. No API Key, signing credential, or user screenshot was added to the repository.

| Boundary | Evidence | Result |
|---|---|---|
| Credentials | `SystemCredentialStore` uses OS keyring; SQLite stores only `credential_ref`; model summaries expose `has_api_key` only. `model_config` and `foundation` tests verify secrets are absent from DB and serialized projections. | Pass |
| Endpoint transport | `validate_endpoint` rejects URL credentials/fragments and non-loopback HTTP. `reqwest` uses native TLS and `Policy::none()`; redirects are also rejected after response. | Pass |
| Provider errors | Provider response bodies are used only for local classification and are not returned or logged; public errors contain stable codes and short Chinese messages. | Pass |
| Logs | Runtime logging records error codes rather than request bodies. Export scans only `.log` files from `app_log_dir` and applies `sanitize_log_line`; tests cover Bearer, `api_key`, `x-api-key`, and `raw_response`. | Pass |
| Model output | Result and history use React text children inside `<pre>`; there is no `dangerouslySetInnerHTML`, Markdown renderer, remote image loader, or script execution path. A frontend test verifies script-shaped output remains text. | Pass |
| Tauri capabilities | `capabilities/default.json` grants only `core:default` to named application windows. Privileged clipboard, dialog, shortcut, autostart, filesystem, keyring, DB, and network operations stay in Rust commands. | Pass |
| File writes | SQLite writes only below `app_data_dir`. Log export writes only to the path returned by the native save dialog; no IPC command accepts an arbitrary filesystem path. | Pass |
| Database integrity | Foreign keys, secure delete, full synchronous writes, trusted schema off, transactions, and image/history cascading deletes are enabled and covered by integration tests. | Pass |
| Screenshot limits | Input is decoded and re-encoded as PNG, capped at 8000 px and 8 MiB Base64 before provider submission. | Pass |
| Web content | The app loads bundled frontend assets only. CSP restricts default sources to self and allows only Tauri IPC/asset schemes plus data/blob images; provider networking occurs in Rust. | Pass |

## Additional checks

- `cargo clippy --all-targets -j 1 -- -D warnings`: passed.
- `cargo test -j 1`: passed, including credential, log redaction, URL, provider, history, and corruption tests.
- `npm run lint` and `npm test -- --run`: passed.
- The WebdriverIO browser fallback bridge is created only when Tauri internals are absent. In the packaged application Tauri supplies that object before frontend code; the bridge has no filesystem, network, keyring, or Rust command capability.

## Open release risks

- `npm install` reported 3 dependency advisories (2 moderate, 1 high). Detailed `npm audit` was not run because it would transmit the dependency manifest to the npm registry and that egress was not authorized. Release owners must review the advisories in an approved network context and upgrade without `--force` unless breaking changes are understood.
- Application-level encryption is intentionally out of scope. History relies on the current OS account and filesystem protections, as disclosed in the product specification and README.
- macOS Keychain prompts, sandbox/signing behavior, and permissions must still be tested on a signed macOS build.
