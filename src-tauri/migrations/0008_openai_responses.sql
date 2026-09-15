PRAGMA foreign_keys = OFF;

BEGIN;

CREATE TEMP TABLE model_config_app_refs AS
SELECT id, active_model_config_id
FROM app_settings;

CREATE TEMP TABLE model_config_history_refs AS
SELECT id, model_config_id
FROM history_entries;

CREATE TABLE model_configs_new (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    protocol TEXT NOT NULL CHECK (protocol IN ('openai', 'openai-responses', 'anthropic', 'gemini')),
    base_url TEXT NOT NULL,
    model_id TEXT NOT NULL,
    reasoning_effort TEXT CHECK (
        reasoning_effort IS NULL
        OR reasoning_effort IN ('none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max')
    ),
    api_key TEXT,
    credential_ref TEXT,
    test_status TEXT NOT NULL DEFAULT 'untested' CHECK (test_status IN ('untested', 'passed', 'failed')),
    tested_at TEXT,
    test_error_code TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT INTO model_configs_new (
    id, name, protocol, base_url, model_id, reasoning_effort, api_key, credential_ref,
    test_status, tested_at, test_error_code, created_at, updated_at
)
SELECT
    id, name, protocol, base_url, model_id, reasoning_effort, api_key, credential_ref,
    test_status, tested_at, test_error_code, created_at, updated_at
FROM model_configs;

DROP TABLE model_configs;
ALTER TABLE model_configs_new RENAME TO model_configs;

UPDATE app_settings
SET active_model_config_id = (
    SELECT active_model_config_id
    FROM model_config_app_refs
    WHERE model_config_app_refs.id = app_settings.id
);

UPDATE history_entries
SET model_config_id = (
    SELECT model_config_id
    FROM model_config_history_refs
    WHERE model_config_history_refs.id = history_entries.id
);

DROP TABLE model_config_app_refs;
DROP TABLE model_config_history_refs;

COMMIT;

PRAGMA foreign_keys = ON;
