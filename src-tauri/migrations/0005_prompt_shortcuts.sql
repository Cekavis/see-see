ALTER TABLE prompt_presets ADD COLUMN capture_shortcut TEXT;
CREATE UNIQUE INDEX idx_prompt_capture_shortcut ON prompt_presets(capture_shortcut) WHERE capture_shortcut IS NOT NULL;
UPDATE app_settings SET active_prompt_id = NULL, capture_shortcut = '';
