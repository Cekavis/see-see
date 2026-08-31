# Quickstart: Concurrent Analysis Requests

1. Start the app with one valid model and prompt configuration.
2. Start a screenshot analysis and leave it streaming.
3. Start a second screenshot analysis before the first completes.
4. Confirm two result windows exist, each shows its own model/prompt names, and interleaved output is not mixed.
5. Close or cancel the first window and confirm the second continues.
6. Force a request failure, change the active model or prompt, and press **重试** in the failed window.
7. Confirm the retry stays in that window/run and displays the original request configuration.

Automated checks:

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm test`
- `npm run build`
