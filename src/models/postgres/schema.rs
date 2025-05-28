// @generated automatically by Diesel CLI.

diesel::table! {
    batteries (series_id, time) {
        series_id -> Int4,
        time -> Timestamp,
        charge_wh -> Int4,
        energy_in_wh -> Int8,
        energy_out_wh -> Int8,
        power_w -> Int4,
    }
}

diesel::table! {
    bidir_meters (series_id, time) {
        series_id -> Int4,
        time -> Timestamp,
        energy_in_wh -> Int8,
        energy_out_wh -> Int8,
        power_w -> Int4,
    }
}

diesel::table! {
    generators (series_id, time) {
        series_id -> Int4,
        time -> Timestamp,
        energy_wh -> Int8,
        power_w -> Int4,
        runtime_s -> Int8,
    }
}

diesel::table! {
    heatpumps (series_id, time) {
        series_id -> Int4,
        time -> Timestamp,
        energy_wh -> Int8,
        power_w -> Int4,
        heat_wh -> Int8,
        cold_wh -> Int8,
        defrost_wh -> Int8,
        cop_pct -> Int2,
        boiler_top_degc_e1 -> Nullable<Int2>,
        boiler_mid_degc_e1 -> Nullable<Int2>,
        boiler_bot_degc_e1 -> Nullable<Int2>,
    }
}

diesel::table! {
    simple_meters (series_id, time) {
        series_id -> Int4,
        time -> Timestamp,
        energy_wh -> Int8,
        power_w -> Int4,
    }
}

diesel::table! {
    weathers (series_id, time) {
        series_id -> Int4,
        time -> Timestamp,
        temp_in_degc_e1 -> Int2,
        hum_in_e3 -> Int2,
        temp_out_degc_e1 -> Nullable<Int2>,
        hum_out_e3 -> Nullable<Int2>,
        rain_day_um -> Nullable<Int4>,
        rain_act_um -> Nullable<Int4>,
        rain_acc_um -> Int8,
        wind_act_mms -> Nullable<Int4>,
        wind_gust_mms -> Nullable<Int4>,
        wind_dir_deg_e1 -> Nullable<Int2>,
        baro_sea_pa -> Int4,
        baro_abs_pa -> Int4,
        uv_index_e1 -> Nullable<Int2>,
        dew_point_degc_e1 -> Nullable<Int2>,
        temp_x1_degc_e1 -> Nullable<Int2>,
        hum_x1_e3 -> Nullable<Int2>,
        temp_x2_degc_e1 -> Nullable<Int2>,
        hum_x2_e3 -> Nullable<Int2>,
        temp_x3_degc_e1 -> Nullable<Int2>,
        hum_x3_e3 -> Nullable<Int2>,
        temp_x4_degc_e1 -> Nullable<Int2>,
        hum_x4_e3 -> Nullable<Int2>,
        temp_x5_degc_e1 -> Nullable<Int2>,
        hum_x5_e3 -> Nullable<Int2>,
        temp_x6_degc_e1 -> Nullable<Int2>,
        hum_x6_e3 -> Nullable<Int2>,
        temp_x7_degc_e1 -> Nullable<Int2>,
        hum_x7_e3 -> Nullable<Int2>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    batteries,
    bidir_meters,
    generators,
    heatpumps,
    simple_meters,
    weathers,
);
