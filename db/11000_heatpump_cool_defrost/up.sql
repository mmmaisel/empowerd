ALTER TABLE heatpumps
    ADD COLUMN cold_wh BIGINT,
    ADD COLUMN defrost_wh BIGINT;

UPDATE heatpumps
    SET cop_pct = 0 WHERE cop_pct IS NULL AND power_w = 0;
UPDATE heatpumps
    SET cop_pct = 400 WHERE cop_pct IS NULL AND power_w != 0;
UPDATE heatpumps
    SET heat_wh = 0 WHERE heat_wh IS NULL;
UPDATE heatpumps SET cold_wh = 0, defrost_wh = 0;

ALTER TABLE heatpumps
    ALTER COLUMN cop_pct SET NOT NULL,
    ALTER COLUMN heat_wh SET NOT NULL,
    ALTER COLUMN cold_wh SET NOT NULL,
    ALTER COLUMN defrost_wh SET NOT NULL;
