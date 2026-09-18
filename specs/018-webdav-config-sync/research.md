# Research: WebDAV 配置同步

## Existing project patterns

- `src/views/DesktopSettings.tsx` is the existing 常规页 settings group and already handles loading, retry notifications, and persisted settings through a typed API facade.
- `src/ipc.ts` defines camelCase frontend contracts over Tauri commands; new WebDAV operations should follow the same shape.
- `src-tauri/src/settings.rs` owns SQLite-backed settings and model/prompt validation. It is the right place for WebDAV setting persistence and atomic config import/export helpers.
- `src-tauri/src/credentials.rs` already abstracts the platform credential store and is shared by Windows and macOS. `AppState` currently receives that store during startup and will retain it for WebDAV password access.
- `reqwest` is already available with native TLS and system proxy support, so no new HTTP dependency is needed.

## Decisions

### WebDAV protocol surface

Use standard HTTP methods against one application-owned JSON file:

- `MKCOL` for each missing path segment of the configured remote root.
- `PUT` to replace the snapshot file.
- `GET` to download the snapshot file.
- HTTP Basic authentication with the remembered WebDAV username/password.

The backend maps authentication failures, missing files, invalid responses, and transport failures to existing `AppError` categories so the UI can reuse the current notification pattern.

### Endpoint and path safety

Reuse the project rule that remote endpoints require HTTPS, with HTTP allowed only for loopback hosts. Reject query/fragment-bearing WebDAV endpoints and path traversal segments. Normalize an empty remote root to `see-see`, trim surrounding slashes, and append a fixed filename such as `see-see-config.json` below that root.

### Snapshot contents and credentials

The snapshot is versioned JSON with model metadata, prompt name/body, and the active model identifier. It intentionally omits prompt shortcuts, model API Keys, WebDAV credentials, history, and unrelated desktop settings. API Keys stay local and must be entered on each platform.

### Download merge behavior

Parse and validate the whole document before opening a database transaction. Match an incoming model or prompt by ID first and case-insensitive name second. Update matched local rows, add unmatched rows, preserve local prompt shortcuts, and leave local-only rows untouched. Resolve the active model to the matched local row only when it is present in the incoming snapshot.

### Persistence of WebDAV settings

Store non-secret fields in `app_settings` and use one stable keyring entry for the single WebDAV password. The UI receives only `hasPassword`; an empty password input retains the saved value, while an explicit clear action deletes it.

## Alternatives considered

- **Sync API Keys**: rejected because repository security rules prohibit exporting credentials and it would expose provider secrets to the WebDAV account.
- **Add a new WebDAV crate**: rejected because existing `reqwest` supports custom methods, Basic auth, proxies, and TLS already used by the application.
- **Replace all local configs on download**: rejected because it could delete device-specific configs and is not needed for cross-platform merge.
- **Put the password in SQLite**: rejected because the project already has a system credential store abstraction and the password should not be persisted as application data.
