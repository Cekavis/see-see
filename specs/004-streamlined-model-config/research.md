# Research: Streamlined Model Configuration

## Decision 1: Add a same-row nullable API-key column

- **Decision**: Add a nullable `api_key` field to `model_configs` and make it the sole runtime source for model authentication.
- **Rationale**: It directly satisfies the request that the endpoint and key be stored together and simplifies CRUD, edit, copy, test, and analysis flows.
- **Alternatives considered**: Keeping a credential reference contradicts the requirement; storing a separate plaintext table still separates the key from the endpoint; reversible obfuscation adds complexity without meaningful security.

## Decision 2: Retain legacy columns only as an upgrade bridge

- **Decision**: Add the new column without rebuilding the existing table, clear old test fields, stop reading/writing test fields, and keep `credential_ref` only long enough to migrate a readable system credential on startup.
- **Rationale**: Additive SQLite migration is robust under the repository's current lightweight initialization scheme and avoids risky foreign-key table reconstruction. Runtime behavior no longer records test results even if obsolete nullable columns remain.
- **Alternatives considered**: Rebuilding the table would remove obsolete columns but requires carefully disabling and restoring foreign keys around references from `app_settings`; leaving test results populated would conflict with the new semantics.

## Decision 3: Make legacy credential migration best-effort, two-phase, and one-way

- **Decision**: At app-state initialization, inspect legacy references. If a credential is readable, first commit its exact value to the row while retaining the reference, then delete the system credential, and only then clear the reference. Missing entries become keyless; unexpected credential-store errors leave the reference for a future retry but never block startup. Rows that already have both values retry only cleanup.
- **Rationale**: Existing users retain readable keys without being locked out by platform keyring failures. The order is idempotent and prevents crashes or deletion failures from losing the only copy.
- **Alternatives considered**: Failing startup on a keyring error violates recoverability; deleting first risks key loss; silently abandoning every reference would unnecessarily discard available keys.

## Decision 4: Keep saved keys out of webview summaries

- **Decision**: Model list/edit summaries continue to expose only `hasApiKey`. Editing leaves the masked key field blank and preserves the saved same-row key unless the user replaces or explicitly clears it. Copying happens in Rust. Logs, history, notifications, provider errors, and sanitized exports keep existing redaction boundaries.
- **Rationale**: Plain-text at-rest storage does not require transporting saved secrets into the webview. The configuration stays fully usable while disclosure remains minimized.
- **Alternatives considered**: Returning the key in list responses unnecessarily widens exposure; rendering the key on cards increases shoulder-surfing risk without helping the task.

## Decision 5: Treat testing as a pure draft operation

- **Decision**: `test_model_config` validates and sends the current connection draft, resolving an existing same-row key by draft ID only when the edit field is blank. It returns a transient result and performs no database write. The UI does not save or activate before or after the call.
- **Rationale**: This matches the mental model of a diagnostic check and allows both unsaved testing and untested saving.
- **Alternatives considered**: Testing a saved ID creates lifecycle coupling; recording pass/fail recreates stale status; automatically activating on success conflates diagnosis with selection.

## Decision 6: Remove the activation test gate

- **Decision**: Activation requires only that the referenced configuration exists.
- **Rationale**: Saving and testing are independent, so a persisted pass status cannot be a valid prerequisite.
- **Alternatives considered**: Re-testing during activation adds latency/cost and still couples selection to network availability; keeping an in-memory pass gate would make activation inconsistent across reloads.

## Decision 7: Copy immediately using the existing prompt-copy pattern

- **Decision**: Add a model-copy command that creates a saved inactive sibling, copies every connection field including the key, and generates `副本`, `副本 (2)`, and later unique names within the 80-character limit.
- **Rationale**: It gives a one-click action consistent with prompt cards and avoids opening an editor just to save identical values.
- **Alternatives considered**: Prefilling an unsaved editor requires an extra save and makes “copy” ambiguous; copying without the key does not create a reusable connection.

## Decision 8: Use a nullable form as the editor state machine

- **Decision**: Represent a closed editor as no form value, an add editor as a default draft, and an edit editor as a draft copied from a saved summary. Save and cancel close it; failed operations retain it.
- **Rationale**: A single state prevents hidden stale fields and maps directly to what is rendered.
- **Alternatives considered**: A separate visibility boolean can drift from form content; keeping an always-mounted hidden form retains stale DOM state and complicates accessibility.

## Decision 9: Remove only generic page-level subtitles

- **Decision**: Remove descriptive paragraphs immediately beneath the main titles for General, Models, Prompts, History, About, and the onboarding welcome card. Keep result progress, permission recovery, step readiness, field validation, privacy, fee, destructive-action, and empty-state guidance.
- **Rationale**: This reduces repeated prose while preserving information needed for decisions and recovery.
- **Alternatives considered**: Removing every hint would damage usability and safety; removing structural section headings would weaken navigation and accessibility.
