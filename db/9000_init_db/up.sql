CREATE TABLE batteries (
    series_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    charge_wh INTEGER NOT NULL,
    energy_in_wh BIGINT NOT NULL,
    energy_out_wh BIGINT NOT NULL,
    power_w INTEGER NOT NULL,
    PRIMARY KEY(series_id, time)
);

CREATE TABLE bidir_meters (
    series_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    energy_in_wh BIGINT NOT NULL,
    energy_out_wh BIGINT NOT NULL,
    power_w INTEGER NOT NULL,
    PRIMARY KEY(series_id, time)
);

CREATE TABLE generators (
    series_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    energy_wh BIGINT NOT NULL,
    power_w INTEGER NOT NULL,
    runtime_s BIGINT NOT NULL,
    PRIMARY KEY(series_id, time)
);

CREATE TABLE heatpumps (
    series_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    energy_wh BIGINT NOT NULL,
    power_w INTEGER NOT NULL,
    heat_wh BIGINT,
    cop_pct SMALLINT,
    boiler_top_degc_e1 SMALLINT,
    boiler_mid_degc_e1 SMALLINT,
    boiler_bot_degc_e1 SMALLINT,
    PRIMARY KEY(series_id, time)
);

CREATE TABLE simple_meters (
    series_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    energy_wh BIGINT NOT NULL,
    power_w INTEGER NOT NULL,
    PRIMARY KEY(series_id, time)
);

CREATE TABLE weathers (
    series_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    temp_in_degc_e1 SMALLINT NOT NULL,
    hum_in_e3 SMALLINT NOT NULL,
    temp_out_degc_e1 SMALLINT,
    hum_out_e3 SMALLINT,
    rain_day_um INTEGER NOT NULL,
    rain_act_um INTEGER NOT NULL,
    wind_act_mms INTEGER NOT NULL,
    wind_gust_mms INTEGER NOT NULL,
    wind_dir_deg_e1 SMALLINT NOT NULL,
    baro_sea_pa INTEGER NOT NULL,
    baro_abs_pa INTEGER NOT NULL,
    uv_index_e1 SMALLINT NOT NULL,
    dew_point_degc_e1 SMALLINT,
    temp_x1_degc_e1 SMALLINT,
    hum_x1_e3 SMALLINT,
    temp_x2_degc_e1 SMALLINT,
    hum_x2_e3 SMALLINT,
    temp_x3_degc_e1 SMALLINT,
    hum_x3_e3 SMALLINT,
    PRIMARY KEY(series_id, time)
);
