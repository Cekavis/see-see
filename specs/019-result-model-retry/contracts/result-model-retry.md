# Result Model Retry Contract

## IPC

- Command: `retry_analysis_with_model`
- Dispatch: asynchronous Tauri command, required for responsive native Windows WebView creation.
- Arguments: `{ runId: string, modelConfigId: string }`
- Response: `{ runId: string }` with a new run identity after result-window creation.
- Errors: existing structured `AppError`; no mutation of source on failure.
- Load model/key by selected ID; reuse frozen prompt/image/history policy; do not update active model.
- Existing `retry_analysis` remains unchanged.

## Chooser

`ModelRetryDialog` props:

- `open: boolean`
- `loadModels: () => Promise<ModelConfigSummary[]>`
- `onSelect: (modelConfigId: string) => Promise<unknown>`
- `onClose: () => void`

Load on opening; selection invokes once while pending; on success close; on failure stay open with recoverable error. Retain existing order. Native modal uses an accessible title and list label, arrow/Home/End navigation, Escape, focus restoration, and native keyboard activation. No search, model count, or loading/creation progress text.

Rows display only the saved configuration name in a compact single line. Ellipsis bounds long names, with the full name retained in the accessible label and title. Select using configuration IDs even when names repeat.

## Result wiring

Optional `onLoadModels` and `onRetryWithModel` props expose the footer action in Result. App supplies `ipc.listModelConfigs` and a callback carrying the source run ID. Source Result snapshot is never replaced by the new run.

Result subscription cleanup invalidates its channel callback and pending attach responses, including errors, so StrictMode effect replay cannot duplicate streaming output.
