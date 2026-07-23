# US2 validation

Date: 2026-07-23

- Remote HTTP is rejected and HTTPS plus localhost/loopback HTTP are accepted by `foundation.rs`.
- Wrong credentials map to `auth_failed`; a wiremock request count of one verifies no automatic retry.
- A provider response indicating unsupported image input maps to `image_not_supported`.
- Model CRUD tests cover unique names, test-state reset, active constraints, credential deletion, and key clearing.
- Serialized model summaries expose only `hasApiKey`; request JSON and SQLite never contain the API key. Provider response bodies and secrets are not logged.
- Frontend tests cover manual model ID fallback, model-list failure recovery, connection-test cost notice, classified failure display, and clearing the key field after save.
