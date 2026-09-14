ALTER TABLE model_configs ADD COLUMN reasoning_effort TEXT
    CHECK (reasoning_effort IS NULL OR reasoning_effort IN ('low', 'medium', 'high'));

UPDATE model_configs
SET reasoning_effort = 'low'
WHERE protocol = 'openai' AND reasoning_effort IS NULL;
