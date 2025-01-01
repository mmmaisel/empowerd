import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { PowerConsumptionPlot } from "./PowerConsumptionPlot";

test("Query for single sources", () => {
    const queries = new PowerConsumptionPlot({
        ...BackendConfigDefault,
        heatpumps: [1],
        wallboxes: [2],
    }).queries();

    // prettier-ignore
    const heatpump_sql =
        "SELECT time, power_w AS \"heatpump.power_w\" FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    // prettier-ignore
    const wallbox_sql =
        "SELECT time, power_w AS \"wallbox.power_w\" FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(heatpump_sql);
    expect(queries[1].rawSql).toBe(wallbox_sql);
});

test("Query for dual sources", () => {
    const queries = new PowerConsumptionPlot({
        ...BackendConfigDefault,
        heatpumps: [1, 2],
        wallboxes: [3, 4],
    }).queries();

    // prettier-ignore
    const heatpump_sql =
        "WITH heatpump1 AS (" +
            "SELECT time, power_w FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), heatpump2 AS (" +
            "SELECT time, power_w FROM heatpumps " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"heatpump.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(heatpump1.time, heatpump2.time) AS time, " +
            "COALESCE(heatpump1.power_w, 0)+COALESCE(heatpump2.power_w, 0) " +
                "AS \"heatpump.power_w\" " +
            "FROM heatpump1 " +
            "FULL OUTER JOIN heatpump2 ON heatpump1.time = heatpump2.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const wallbox_sql =
        "WITH wallbox3 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), wallbox4 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"wallbox.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(wallbox3.time, wallbox4.time) AS time, " +
            "COALESCE(wallbox3.power_w, 0)+COALESCE(wallbox4.power_w, 0) " +
                "AS \"wallbox.power_w\" " +
            "FROM wallbox3 " +
            "FULL OUTER JOIN wallbox4 ON wallbox3.time = wallbox4.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(heatpump_sql);
    expect(queries[1].rawSql).toBe(wallbox_sql);
});

test("Query consumption", () => {
    const queries = new PowerConsumptionPlot({
        ...BackendConfigDefault,
        batteries: [4],
        generators: [3],
        heatpumps: [6],
        meters: [5],
        solars: [1, 2],
        wallboxes: [7],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "WITH meter5 AS (" +
            "SELECT time, power_w FROM bidir_meters " +
            "WHERE series_id = 5 AND $__timeFilter(time)" +
        "), battery4 AS (" +
            "SELECT time, -power_w AS npower_w FROM batteries " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        "), generator3 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), heatpump6 AS (" +
            "SELECT time, power_w FROM heatpumps " +
            "WHERE series_id = 6 AND $__timeFilter(time)" +
        "), solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), wallbox7 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"consumption.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(meter5.time, battery4.time, generator3.time, " +
                "heatpump6.time, solar1.time, solar2.time, wallbox7.time) " +
                "AS time, " +
            "COALESCE(meter5.power_w, 0)+COALESCE(battery4.npower_w, 0)+" +
            "COALESCE(generator3.power_w, 0)+COALESCE(solar1.power_w, 0)+" +
            "COALESCE(solar2.power_w, 0)" +
            "-COALESCE(heatpump6.power_w, 0)-COALESCE(wallbox7.power_w, 0) " +
                "AS \"consumption.power_w\" " +
            "FROM meter5 " +
            "FULL OUTER JOIN battery4 ON meter5.time = battery4.time " +
            "FULL OUTER JOIN generator3 ON meter5.time = generator3.time " +
            "FULL OUTER JOIN heatpump6 ON meter5.time = heatpump6.time " +
            "FULL OUTER JOIN solar1 ON meter5.time = solar1.time " +
            "FULL OUTER JOIN solar2 ON meter5.time = solar2.time " +
            "FULL OUTER JOIN wallbox7 ON meter5.time = wallbox7.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[2].rawSql).toBe(expected_sql);
});
