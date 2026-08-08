BEGIN;

ALTER TABLE history_entries
ADD COLUMN prompt_config_id TEXT REFERENCES prompt_presets(id) ON DELETE SET NULL;

ALTER TABLE history_entries
ADD COLUMN model_config_id TEXT REFERENCES model_configs(id) ON DELETE SET NULL;

UPDATE history_entries
SET prompt_config_id = (
    SELECT id FROM prompt_presets WHERE name = history_entries.prompt_name COLLATE NOCASE
)
WHERE prompt_config_id IS NULL;

UPDATE history_entries
SET model_config_id = (
    SELECT id FROM model_configs WHERE name = history_entries.model_config_name COLLATE NOCASE
)
WHERE model_config_id IS NULL;

COMMIT;
