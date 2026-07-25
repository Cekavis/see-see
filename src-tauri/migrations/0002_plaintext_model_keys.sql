ALTER TABLE model_configs ADD COLUMN api_key TEXT;

UPDATE model_configs
SET test_status = 'untested', tested_at = NULL, test_error_code = NULL;
