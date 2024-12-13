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

    expect(queries[0].rawSql).toBe(solar_sql);
    expect(queries[1].rawSql).toBe(generator_sql);
    expect(queries[2].rawSql).toBe(battery_sql);
    expect(queries[3].rawSql).toBe(meter_sql);
});

test("Query for dual sources", () => {
    const queries = new PowerStats(
        {
            ...BackendConfigDefault,
            solars: [1, 2],
            generators: [3, 4],
            batteries: [5, 6],
            meters: [7, 8],
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

    expect(queries[0].rawSql).toBe(solar_sql);
    expect(queries[1].rawSql).toBe(generator_sql);
    expect(queries[2].rawSql).toBe(battery_sql);
    expect(queries[3].rawSql).toBe(meter_sql);
});
