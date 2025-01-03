ALTER TABLE heatpumps
    ALTER COLUMN cop_pct DROP NOT NULL,
    ALTER COLUMN heat_wh DROP NOT NULL,
    DROP COLUMN cold_wh,
    DROP COLUMN defrost_wh;
