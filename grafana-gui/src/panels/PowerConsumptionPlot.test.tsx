import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { PowerConsumptionPlot } from "./PowerConsumptionPlot";

test("Query for single heatpump source", () => {
    const queries = new PowerConsumptionPlot({
        ...BackendConfigDefault,
        heatpumps: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, power_w AS \"heatpump.power_w\" FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual heatpump source", () => {
    const queries = new PowerConsumptionPlot({
        ...BackendConfigDefault,
        heatpumps: [1, 2],
        generators: [],
    }).queries();

    // prettier-ignore
    const expected_sql =
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

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query consumption", () => {
    const queries = new PowerConsumptionPlot({
        ...BackendConfigDefault,
        solars: [1, 2],
        generators: [3],
        batteries: [4],
        meters: [5],
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
        "), solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"consumption.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(meter5.time, battery4.time, generator3.time, " +
                "solar1.time, solar2.time) AS time, " +
            "COALESCE(meter5.power_w, 0)+COALESCE(battery4.npower_w, 0)+" +
            "COALESCE(generator3.power_w, 0)+COALESCE(solar1.power_w, 0)+" +
            "COALESCE(solar2.power_w, 0) " +
                "AS \"consumption.power_w\" " +
            "FROM meter5 " +
            "FULL OUTER JOIN battery4 ON meter5.time = battery4.time " +
            "FULL OUTER JOIN generator3 ON meter5.time = generator3.time " +
            "FULL OUTER JOIN solar1 ON meter5.time = solar1.time " +
            "FULL OUTER JOIN solar2 ON meter5.time = solar2.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[2].rawSql).toBe(expected_sql);
});
