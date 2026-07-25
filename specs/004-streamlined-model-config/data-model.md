# Data Model: Streamlined Model Configuration

## Model Configuration

Represents one complete local provider connection.

| Field | Meaning | Validation |
|-------|---------|------------|
| `id` | Stable generated identity | Non-empty unique identifier |
| `name` | User-facing name | Trimmed, 1–80 characters, case-insensitively unique |
| `protocol` | Provider request shape | OpenAI, Anthropic, or Gemini |
| `baseUrl` | Provider endpoint | Trimmed, trailing slash removed; HTTPS except localhost/loopback HTTP |
| `modelId` | Provider model identifier | Trimmed, 1–200 characters |
| `apiKey` | Optional plain-text authentication secret | Exact entered value; empty input becomes absent |
| `createdAt` | Creation time | Generated on first save |
| `updatedAt` | Last configuration change | Updated on save |
| `isActive` | Derived current-selection flag | At most one configuration is current |

Relationships:

- `App Settings.activeModelConfigId` may reference one Model Configuration.
- History retains only configuration/model display snapshots and never the API key.
- A copied Model Configuration has a new identity and name but no persistent relationship to its source.

State transitions:

```text
absent -> saved -> edited -> saved
saved -> copied sibling
saved <-> current
saved -> deleted
```

Testing is not a state transition and adds no fields to this entity.

## Configuration Draft

Temporary frontend values for add/edit, model listing, and connection testing.

| Field | Meaning |
|-------|---------|
| `id` | Present only when editing an existing configuration |
| `name` | Draft display name used on save |
| `protocol` | Draft provider protocol |
| `baseUrl` | Draft endpoint |
| `modelId` | Draft model identifier |
| `apiKey` | Optional replacement key; blank preserves an existing saved key unless clear is selected |
| `clearApiKey` | Explicit removal intent available only while editing |

State transitions:

```text
closed -> adding -> closed (save or cancel)
closed -> editing -> closed (save or cancel)
adding/editing -> adding/editing (failed save, model listing, or test)
```

The draft is discarded on cancel, successful save, navigation unmount, or deletion of the edited record.

## Connection Test Result

Transient response returned to the caller only.

| Field | Meaning |
|-------|---------|
| `passed` | Whether the test request produced the expected usable response |
| `latencyMs` | Observed test duration for immediate feedback |
| `error` | Optional categorized, sanitized failure |

The result has no identifier, timestamp, or persisted relationship.

## Legacy Credential Reference

Compatibility-only field on databases created by earlier versions.

- If `apiKey` is absent and the reference resolves, its exact value is copied to `apiKey` while the reference remains, then the legacy credential is deleted and the reference is cleared.
- If the reference is missing, it is cleared and the model remains keyless.
- If the credential provider errors, startup continues and the reference remains for retry.
- Runtime connection, editing, copying, and testing never consult this field after migration.
