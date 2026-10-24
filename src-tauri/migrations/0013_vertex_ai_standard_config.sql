ALTER TABLE model_configs ADD COLUMN project_id TEXT;
ALTER TABLE model_configs ADD COLUMN location TEXT;

UPDATE model_configs
SET location = 'global'
WHERE protocol = 'vertex-ai' AND location IS NULL;
