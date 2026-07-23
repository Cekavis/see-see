CREATE TABLE IF NOT EXISTS prompt_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    body TEXT NOT NULL,
    is_builtin INTEGER NOT NULL CHECK (is_builtin IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS model_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
    protocol TEXT NOT NULL CHECK (protocol IN ('openai', 'anthropic', 'gemini')),
    base_url TEXT NOT NULL,
    model_id TEXT NOT NULL,
    credential_ref TEXT,
    test_status TEXT NOT NULL DEFAULT 'untested' CHECK (test_status IN ('untested', 'passed', 'failed')),
    tested_at TEXT,
    test_error_code TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    active_model_config_id TEXT REFERENCES model_configs(id) ON DELETE SET NULL,
    active_prompt_id TEXT REFERENCES prompt_presets(id) ON DELETE SET NULL,
    capture_shortcut TEXT NOT NULL,
    save_history INTEGER NOT NULL CHECK (save_history IN (0, 1)),
    autostart INTEGER NOT NULL CHECK (autostart IN (0, 1)),
    result_always_on_top INTEGER NOT NULL CHECK (result_always_on_top IN (0, 1)),
    onboarding_completed INTEGER NOT NULL DEFAULT 0 CHECK (onboarding_completed IN (0, 1)),
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS history_entries (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL CHECK (status IN ('success', 'failed')),
    result_text TEXT,
    error_code TEXT,
    error_message TEXT,
    prompt_name TEXT NOT NULL,
    prompt_body TEXT NOT NULL,
    model_config_name TEXT NOT NULL,
    protocol TEXT NOT NULL,
    model_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    CHECK (
        (status = 'success' AND result_text IS NOT NULL AND length(trim(result_text)) > 0 AND error_code IS NULL)
        OR
        (status = 'failed' AND result_text IS NULL AND error_code IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS history_images (
    history_id TEXT PRIMARY KEY REFERENCES history_entries(id) ON DELETE CASCADE,
    mime_type TEXT NOT NULL CHECK (mime_type = 'image/png'),
    width INTEGER NOT NULL CHECK (width BETWEEN 1 AND 8000),
    height INTEGER NOT NULL CHECK (height BETWEEN 1 AND 8000),
    original_bytes BLOB NOT NULL,
    thumbnail_bytes BLOB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_history_started ON history_entries(started_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_history_status ON history_entries(status);
CREATE INDEX IF NOT EXISTS idx_history_prompt ON history_entries(prompt_name);

INSERT OR IGNORE INTO prompt_presets (id, name, body, is_builtin, created_at, updated_at) VALUES
('00000000-0000-4000-8000-000000000001', '通用翻译为中文', '请识别图片中的文字并完整翻译为简体中文。只输出原文和译文，使用空行分隔。', 1, '2026-07-23T00:00:00Z', '2026-07-23T00:00:00Z'),
('00000000-0000-4000-8000-000000000002', '日语学习解析', '你是一个日语老师，一位日语初学者需要理解图片中的日文内容，请依次输出日文原文、完整中文翻译、逐词假名注音/罗马音/含义和简要语法说明，各部分用空行分隔，不输出其他内容。', 1, '2026-07-23T00:00:00Z', '2026-07-23T00:00:00Z');

INSERT OR IGNORE INTO app_settings (
    id, active_model_config_id, active_prompt_id, capture_shortcut,
    save_history, autostart, result_always_on_top, updated_at
) VALUES (
    1, NULL, '00000000-0000-4000-8000-000000000002', 'Alt+Shift+A',
    1, 0, 1, '2026-07-23T00:00:00Z'
);
