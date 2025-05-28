ALTER TABLE weathers
    ADD COLUMN rain_acc_um BIGINT;

UPDATE weathers
    SET rain_acc_um = 0 WHERE rain_acc_um IS NULL;

ALTER TABLE weathers
    ALTER COLUMN rain_acc_um SET NOT NULL;
