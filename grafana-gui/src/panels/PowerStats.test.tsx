import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { PowerStats } from "./PowerStats";

test("Query for single sources", () => {
    const queries = new PowerStats(
        {
            ...BackendConfigDefault,
            solars: [1],
            generators: [2],
            batteries: [3],
            meters: [4],
            wallboxes: [5],
            heatpumps: [6],
        },
        undefined,
        undefined as any
    ).queries();

    // prettier-ignore
    const solar_sql =
        "SELECT MAX(energy_wh)-MIN(energy_wh) AS \"solar.energy_wh\" " +
        "FROM simple_meters " +
        "WHERE series_id = 1 AND $__timeFilter(time)";

    // prettier-ignore
    const generator_sql =
        "SELECT (MAX(runtime_s)-MIN(runtime_s)) * 0.222222 " +
            "AS \"generator.energy_wh\" " +
        "FROM generators " +
        "WHERE series_id = 2 AND $__timeFilter(time)";

    // prettier-ignore
    const battery_sql =
        "SELECT MAX(energy_in_wh)-MIN(energy_in_wh) " +
                `AS \"battery.d_energy_in_wh\", ` +
            "MAX(energy_out_wh)-MIN(energy_out_wh) " +
                `AS \"battery.d_energy_out_wh\" ` +
        "FROM batteries " +
        "WHERE series_id = 3 AND $__timeFilter(time)";

    // prettier-ignore
    const meter_sql =
        "SELECT MAX(energy_in_wh)-MIN(energy_in_wh) " +
                `AS \"meter.d_energy_in_wh\", ` +
            "MAX(energy_out_wh)-MIN(energy_out_wh) " +
                `AS \"meter.d_energy_out_wh\" ` +
        "FROM bidir_meters " +
        "WHERE series_id = 4 AND $__timeFilter(time)";

    // prettier-ignore
    const heatpump_sql =
        "SELECT MAX(energy_wh)-MIN(energy_wh) AS \"heatpump.energy_wh\" " +
        "FROM heatpumps " +
        "WHERE series_id = 6 AND $__timeFilter(time)";

    // prettier-ignore
    const wallbox_sql =
        "SELECT MAX(energy_wh)-MIN(energy_wh) AS \"wallbox.energy_wh\" " +
        "FROM simple_meters " +
        "WHERE series_id = 5 AND $__timeFilter(time)";

    // prettier-ignore
    const consumption_sql =
        "WITH meter4 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM bidir_meters " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        "), battery3 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM batteries " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator2 AS (" +
            "SELECT time, runtime_s * 0.222222 AS energy_wh FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), heatpump6 AS (" +
            "SELECT time, energy_wh FROM heatpumps " +
            "WHERE series_id = 6 AND $__timeFilter(time)" +
        "), solar1 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), wallbox5 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 5 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(" +
                "MAX(meter4.energy_in_wh)-MIN(meter4.energy_in_wh)" +
                "-MAX(meter4.energy_out_wh)+MIN(meter4.energy_out_wh)" +
            ", 0)+" +
            "COALESCE(" +
                "MAX(battery3.energy_out_wh)-MIN(battery3.energy_out_wh)" +
                "-MAX(battery3.energy_in_wh)+MIN(battery3.energy_in_wh)" +
            ", 0)+" +
            "COALESCE(MAX(generator2.energy_wh)-MIN(generator2.energy_wh), 0)+" +
            "COALESCE(MAX(solar1.energy_wh)-MIN(solar1.energy_wh), 0)" +
            "-COALESCE(MAX(heatpump6.energy_wh)-MIN(heatpump6.energy_wh), 0)" +
            "-COALESCE(MAX(wallbox5.energy_wh)-MIN(wallbox5.energy_wh), 0) " +
            "AS d_energy_wh " +
        "FROM meter4 " +
        "FULL OUTER JOIN battery3 ON meter4.time = battery3.time " +
        "FULL OUTER JOIN generator2 ON meter4.time = generator2.time " +
        "FULL OUTER JOIN heatpump6 ON meter4.time = heatpump6.time " +
        "FULL OUTER JOIN solar1 ON meter4.time = solar1.time " +
        "FULL OUTER JOIN wallbox5 ON meter4.time = wallbox5.time";

    expect(queries[0].rawSql).toBe(solar_sql);
    expect(queries[1].rawSql).toBe(generator_sql);
    expect(queries[2].rawSql).toBe(battery_sql);
    expect(queries[3].rawSql).toBe(meter_sql);
    expect(queries[4].rawSql).toBe(heatpump_sql);
    expect(queries[5].rawSql).toBe(wallbox_sql);
    expect(queries[6].rawSql).toBe(consumption_sql);
});

test("Query for dual sources", () => {
    const queries = new PowerStats(
        {
            ...BackendConfigDefault,
            solars: [1, 2],
            generators: [3, 4],
            batteries: [5, 6],
            meters: [7, 8],
            wallboxes: [9, 10],
            heatpumps: [11, 12],
        },
        undefined,
        undefined as any
    ).queries();

    // prettier-ignore
    const solar_sql =
        "WITH solar1 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(MAX(solar1.energy_wh)-MIN(solar1.energy_wh), 0)+" +
            "COALESCE(MAX(solar2.energy_wh)-MIN(solar2.energy_wh), 0) " +
            "AS \"solar.energy_wh\" " +
        "FROM solar1 " +
        "FULL OUTER JOIN solar2 ON solar1.time = solar2.time";

    // prettier-ignore
    const generator_sql =
        "WITH generator3 AS (" +
            "SELECT time, runtime_s * 0.222222 AS energy_wh FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, runtime_s * 0.222222 AS energy_wh FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(MAX(generator3.energy_wh)-MIN(generator3.energy_wh), 0)+" +
            "COALESCE(MAX(generator4.energy_wh)-MIN(generator4.energy_wh), 0) " +
            "AS \"generator.energy_wh\" " +
        "FROM generator3 " +
        "FULL OUTER JOIN generator4 ON generator3.time = generator4.time";

    // prettier-ignore
    const battery_sql =
        "WITH battery5 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM batteries " +
            "WHERE series_id = 5 AND $__timeFilter(time)" +
        "), battery6 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM batteries " +
            "WHERE series_id = 6 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(" +
                "MAX(battery5.energy_in_wh)-MIN(battery5.energy_in_wh), 0)+" +
            "COALESCE(" +
                "MAX(battery6.energy_in_wh)-MIN(battery6.energy_in_wh), 0) " +
                `AS \"battery.d_energy_in_wh\", ` +
            "COALESCE(" +
                "MAX(battery5.energy_out_wh)-MIN(battery5.energy_out_wh), 0)+" +
            "COALESCE(" +
                "MAX(battery6.energy_out_wh)-MIN(battery6.energy_out_wh), 0) " +
                `AS \"battery.d_energy_out_wh\" ` +
        "FROM battery5 " +
        "FULL OUTER JOIN battery6 ON battery5.time = battery6.time";

    // prettier-ignore
    const meter_sql =
        "WITH meter7 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM bidir_meters " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        "), meter8 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM bidir_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(" +
                "MAX(meter7.energy_in_wh)-MIN(meter7.energy_in_wh), 0)+" +
            "COALESCE(" +
                "MAX(meter8.energy_in_wh)-MIN(meter8.energy_in_wh), 0) " +
                `AS \"meter.d_energy_in_wh\", ` +
            "COALESCE(" +
                "MAX(meter7.energy_out_wh)-MIN(meter7.energy_out_wh), 0)+" +
            "COALESCE(" +
                "MAX(meter8.energy_out_wh)-MIN(meter8.energy_out_wh), 0) " +
                `AS \"meter.d_energy_out_wh\" ` +
        "FROM meter7 " +
        "FULL OUTER JOIN meter8 ON meter7.time = meter8.time";

    // prettier-ignore
    const heatpump_sql =
        "WITH heatpump11 AS (" +
            "SELECT time, energy_wh FROM heatpumps " +
            "WHERE series_id = 11 AND $__timeFilter(time)" +
        "), heatpump12 AS (" +
            "SELECT time, energy_wh FROM heatpumps " +
            "WHERE series_id = 12 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(MAX(heatpump11.energy_wh)-MIN(heatpump11.energy_wh), 0)+" +
            "COALESCE(MAX(heatpump12.energy_wh)-MIN(heatpump12.energy_wh), 0) " +
            "AS \"heatpump.energy_wh\" " +
        "FROM heatpump11 " +
        "FULL OUTER JOIN heatpump12 ON heatpump11.time = heatpump12.time";

    // prettier-ignore
    const wallbox_sql =
        "WITH wallbox9 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 9 AND $__timeFilter(time)" +
        "), wallbox10 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 10 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(MAX(wallbox9.energy_wh)-MIN(wallbox9.energy_wh), 0)+" +
            "COALESCE(MAX(wallbox10.energy_wh)-MIN(wallbox10.energy_wh), 0) " +
            "AS \"wallbox.energy_wh\" " +
        "FROM wallbox9 " +
        "FULL OUTER JOIN wallbox10 ON wallbox9.time = wallbox10.time";

    // prettier-ignore
    const consumption_sql =
        "WITH meter7 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM bidir_meters " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        "), meter8 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM bidir_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        "), battery5 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM batteries " +
            "WHERE series_id = 5 AND $__timeFilter(time)" +
        "), battery6 AS (" +
            "SELECT time, energy_in_wh, energy_out_wh FROM batteries " +
            "WHERE series_id = 6 AND $__timeFilter(time)" +
        "), generator3 AS (" +
            "SELECT time, runtime_s * 0.222222 AS energy_wh FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, runtime_s * 0.222222 AS energy_wh FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        "), heatpump11 AS (" +
            "SELECT time, energy_wh FROM heatpumps " +
            "WHERE series_id = 11 AND $__timeFilter(time)" +
        "), heatpump12 AS (" +
            "SELECT time, energy_wh FROM heatpumps " +
            "WHERE series_id = 12 AND $__timeFilter(time)" +
        "), solar1 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), wallbox9 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 9 AND $__timeFilter(time)" +
        "), wallbox10 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 10 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(" +
                "MAX(meter7.energy_in_wh)-MIN(meter7.energy_in_wh)" +
                "-MAX(meter7.energy_out_wh)+MIN(meter7.energy_out_wh)" +
            ", 0)+" +
            "COALESCE(" +
                "MAX(meter8.energy_in_wh)-MIN(meter8.energy_in_wh)" +
                "-MAX(meter8.energy_out_wh)+MIN(meter8.energy_out_wh)" +
            ", 0)+" +
            "COALESCE(" +
                "MAX(battery5.energy_out_wh)-MIN(battery5.energy_out_wh)" +
                "-MAX(battery5.energy_in_wh)+MIN(battery5.energy_in_wh)" +
            ", 0)+" +
            "COALESCE(" +
                "MAX(battery6.energy_out_wh)-MIN(battery6.energy_out_wh)" +
                "-MAX(battery6.energy_in_wh)+MIN(battery6.energy_in_wh)" +
            ", 0)+" +
            "COALESCE(MAX(generator3.energy_wh)-MIN(generator3.energy_wh), 0)+" +
            "COALESCE(MAX(generator4.energy_wh)-MIN(generator4.energy_wh), 0)+" +
            "COALESCE(MAX(solar1.energy_wh)-MIN(solar1.energy_wh), 0)+" +
            "COALESCE(MAX(solar2.energy_wh)-MIN(solar2.energy_wh), 0)" +
            "-COALESCE(MAX(heatpump11.energy_wh)-MIN(heatpump11.energy_wh), 0)" +
            "-COALESCE(MAX(heatpump12.energy_wh)-MIN(heatpump12.energy_wh), 0)" +
            "-COALESCE(MAX(wallbox9.energy_wh)-MIN(wallbox9.energy_wh), 0)" +
            "-COALESCE(MAX(wallbox10.energy_wh)-MIN(wallbox10.energy_wh), 0) " +
            "AS d_energy_wh " +
        "FROM meter7 " +
        "FULL OUTER JOIN meter8 ON meter7.time = meter8.time " +
        "FULL OUTER JOIN battery5 ON meter7.time = battery5.time " +
        "FULL OUTER JOIN battery6 ON meter7.time = battery6.time " +
        "FULL OUTER JOIN generator3 ON meter7.time = generator3.time " +
        "FULL OUTER JOIN generator4 ON meter7.time = generator4.time " +
        "FULL OUTER JOIN heatpump11 ON meter7.time = heatpump11.time " +
        "FULL OUTER JOIN heatpump12 ON meter7.time = heatpump12.time " +
        "FULL OUTER JOIN solar1 ON meter7.time = solar1.time " +
        "FULL OUTER JOIN solar2 ON meter7.time = solar2.time " +
        "FULL OUTER JOIN wallbox9 ON meter7.time = wallbox9.time " +
        "FULL OUTER JOIN wallbox10 ON meter7.time = wallbox10.time";

    expect(queries[0].rawSql).toBe(solar_sql);
    expect(queries[1].rawSql).toBe(generator_sql);
    expect(queries[2].rawSql).toBe(battery_sql);
    expect(queries[3].rawSql).toBe(meter_sql);
    expect(queries[4].rawSql).toBe(heatpump_sql);
    expect(queries[5].rawSql).toBe(wallbox_sql);
    expect(queries[6].rawSql).toBe(consumption_sql);
});
