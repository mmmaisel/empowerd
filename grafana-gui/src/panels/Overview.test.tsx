import "intersection-observer";

import { privateFunctions } from "./Overview";

test("Query for single solar source", () => {
    const queries = privateFunctions.mkqueries({
        batteries: [],
        generators: [],
        heatpumps: [],
        meters: [],
        solars: [1],
        weathers: [],
    });

    // prettier-ignore
    const expected_sql0 =
        "SELECT time, power_w AS \"solar.power_w\" FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    // prettier-ignore
    const expected_sql1 = "SELECT NULL AS s_power";

    expect(queries[0].rawSql).toBe(expected_sql0);
    expect(queries[1].rawSql).toBe(expected_sql1);
});

test("Query for single weather source", () => {
    const queries = privateFunctions.mkqueries({
        batteries: [],
        generators: [],
        heatpumps: [],
        meters: [],
        solars: [],
        weathers: [1],
    });

    // prettier-ignore
    const expected_sql0 =
        "SELECT time, " +
                "temp_out_degc_e1/10.0 AS temp_out_degc, " +
                "rain_act_um/1000.0 AS rain_act_mm " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[4].rawSql).toBe(expected_sql0);
});

test("Query for single generator source", () => {
    const queries = privateFunctions.mkqueries({
        batteries: [],
        generators: [2],
        heatpumps: [],
        meters: [],
        solars: [],
        weathers: [],
    });

    // prettier-ignore
    const expected_sql0 =
        "SELECT time, power_w AS \"generator.power_w\" FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time) " +
            "ORDER BY time";

    // prettier-ignore
    const expected_sql3 =
        "SELECT time, power_w * 6.93348 AS \"generator.heat_w\" FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql0);
    expect(queries[3].rawSql).toBe(expected_sql3);
});

test("Query for single sources", () => {
    const queries = privateFunctions.mkqueries({
        batteries: [5],
        generators: [2],
        heatpumps: [3],
        meters: [4],
        solars: [1],
        weathers: [],
    });

    // prettier-ignore
    const expected_sql0 =
        "WITH solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), generator2 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"production.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(generator2.time, solar1.time) AS time, " +
            "COALESCE(solar1.power_w, 0)+COALESCE(generator2.power_w, 0) " +
                "AS \"production.power_w\" " +
            "FROM solar1 " +
            "FULL OUTER JOIN generator2 ON solar1.time = generator2.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const expected_sql1 =
        "WITH meter4 AS (" +
            "SELECT time, power_w FROM bidir_meters " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        "), battery5 AS (" +
            "SELECT time, power_w FROM batteries " +
            "WHERE series_id = 5 AND $__timeFilter(time)" +
        "), generator2 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"consumption.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(" +
                "meter4.time, battery5.time, generator2.time, " +
                "solar1.time) AS time, " +
            "COALESCE(meter4.power_w, 0)+COALESCE(-battery5.power_w, 0)+" +
            "COALESCE(generator2.power_w, 0)+COALESCE(solar1.power_w, 0) " +
                "AS \"consumption.power_w\" " +
            "FROM meter4 " +
            "FULL OUTER JOIN battery5 ON meter4.time = battery5.time " +
            "FULL OUTER JOIN generator2 ON meter4.time = generator2.time " +
            "FULL OUTER JOIN solar1 ON meter4.time = solar1.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const expected_sql2 =
        "SELECT time, charge_wh AS \"battery.charge_wh\", " +
            "power_w AS \"battery.power_w\" " +
        "FROM batteries " +
        "WHERE series_id = 5 AND $__timeFilter(time) " +
        "ORDER BY time";

    // prettier-ignore
    const expected_sql3 =
        "WITH generator2 AS (" +
            "SELECT time, power_w * 6.93348 AS heat_w FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), heatpump3 AS (" +
            "SELECT time, power_w * cop_pct / 100.0 AS heat_w " +
            "FROM heatpumps " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, s_heat " +
        "FROM (SELECT " +
            "COALESCE(generator2.time, heatpump3.time) AS time, " +
            "COALESCE(generator2.heat_w, 0)+COALESCE(heatpump3.heat_w, 0) " +
                "AS s_heat " +
            "FROM generator2 " +
            "FULL OUTER JOIN heatpump3 ON generator2.time = heatpump3.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql0);
    expect(queries[1].rawSql).toBe(expected_sql1);
    expect(queries[2].rawSql).toBe(expected_sql2);
    expect(queries[3].rawSql).toBe(expected_sql3);
});

test("Query for dual", () => {
    const queries = privateFunctions.mkqueries({
        batteries: [9, 10],
        generators: [3, 4],
        heatpumps: [5, 6],
        meters: [7, 8],
        solars: [1, 2],
        weathers: [],
    });

    // prettier-ignore
    const expected_sql0 =
        "WITH solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), generator3 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"production.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(" +
                "generator3.time, generator4.time, " +
                "solar1.time, solar2.time) AS time, " +
            "COALESCE(solar1.power_w, 0)+COALESCE(solar2.power_w, 0)+" +
            "COALESCE(generator3.power_w, 0)+COALESCE(generator4.power_w, 0) " +
                "AS \"production.power_w\" " +
            "FROM solar1 " +
            "FULL OUTER JOIN solar2 ON solar1.time = solar2.time " +
            "FULL OUTER JOIN generator3 ON solar1.time = generator3.time " +
            "FULL OUTER JOIN generator4 ON solar1.time = generator4.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const expected_sql1 =
        "WITH meter7 AS (" +
            "SELECT time, power_w FROM bidir_meters " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        "), meter8 AS (" +
            "SELECT time, power_w FROM bidir_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        "), battery9 AS (" +
            "SELECT time, power_w FROM batteries " +
            "WHERE series_id = 9 AND $__timeFilter(time)" +
        "), battery10 AS (" +
            "SELECT time, power_w FROM batteries " +
            "WHERE series_id = 10 AND $__timeFilter(time)" +
        "), generator3 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        "), solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"consumption.power_w\" " +
        "FROM (SELECT " +
            "COALESCE(meter7.time, meter8.time, " +
                "battery9.time, battery10.time, generator3.time, " +
                "generator4.time, solar1.time, solar2.time) AS time, " +
            "COALESCE(meter7.power_w, 0)+COALESCE(meter8.power_w, 0)+" +
            "COALESCE(-battery9.power_w, 0)+COALESCE(-battery10.power_w, 0)+" +
            "COALESCE(generator3.power_w, 0)+COALESCE(generator4.power_w, 0)+" +
            "COALESCE(solar1.power_w, 0)+COALESCE(solar2.power_w, 0) " +
                "AS \"consumption.power_w\" " +
            "FROM meter7 " +
            "FULL OUTER JOIN meter8 ON meter7.time = meter8.time " +
            "FULL OUTER JOIN battery9 ON meter7.time = battery9.time " +
            "FULL OUTER JOIN battery10 ON meter7.time = battery10.time " +
            "FULL OUTER JOIN generator3 ON meter7.time = generator3.time " +
            "FULL OUTER JOIN generator4 ON meter7.time = generator4.time " +
            "FULL OUTER JOIN solar1 ON meter7.time = solar1.time " +
            "FULL OUTER JOIN solar2 ON meter7.time = solar2.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const expected_sql2 =
        "WITH battery9 AS (" +
            "SELECT time, charge_wh, power_w FROM batteries " +
            "WHERE series_id = 9 AND $__timeFilter(time)" +
        "), battery10 AS (" +
            "SELECT time, charge_wh, power_w FROM batteries " +
            "WHERE series_id = 10 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"battery.charge_wh\", \"battery.power_w\" " +
        "FROM (SELECT " +
            "battery9.time AS time, " +
            "COALESCE(battery9.charge_wh, 0)+COALESCE(battery10.charge_wh, 0) " +
                "AS \"battery.charge_wh\", " +
            "COALESCE(battery9.power_w, 0)+COALESCE(battery10.power_w, 0) " +
                "AS \"battery.power_w\" " +
            "FROM battery9 " +
            "FULL OUTER JOIN battery10 ON battery9.time = battery10.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const expected_sql3 =
        "WITH generator3 AS (" +
            "SELECT time, power_w * 6.93348 AS heat_w FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, power_w * 6.93348 AS heat_w FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        "), heatpump5 AS (" +
            "SELECT time, power_w * cop_pct / 100.0 AS heat_w FROM heatpumps " +
            "WHERE series_id = 5 AND $__timeFilter(time)" +
        "), heatpump6 AS (" +
            "SELECT time, power_w * cop_pct / 100.0 AS heat_w FROM heatpumps " +
            "WHERE series_id = 6 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, s_heat " +
        "FROM (SELECT " +
            "COALESCE(generator3.time, generator4.time, " +
                "heatpump5.time, heatpump6.time) AS time, " +
            "COALESCE(generator3.heat_w, 0)+COALESCE(generator4.heat_w, 0)+" +
            "COALESCE(heatpump5.heat_w, 0)+COALESCE(heatpump6.heat_w, 0) " +
                "AS s_heat " +
            "FROM generator3 " +
            "FULL OUTER JOIN generator4 ON generator3.time = generator4.time " +
            "FULL OUTER JOIN heatpump5 ON generator3.time = heatpump5.time " +
            "FULL OUTER JOIN heatpump6 ON generator3.time = heatpump6.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql0);
    expect(queries[1].rawSql).toBe(expected_sql1);
    expect(queries[2].rawSql).toBe(expected_sql2);
    expect(queries[3].rawSql).toBe(expected_sql3);
});
