import "intersection-observer";

import { privateFunctions } from "./HeatPlot";

test("Query for single generator source", () => {
    const queries = privateFunctions.mkqueries({
        generators: [2],
        heatpumps: [],
    });

    // prettier-ignore
    const expected_sql =
        "WITH generator2 AS (" +
            "SELECT time, power_w * (1-0.138)/0.138 * 1.11 AS heat_w " +
            "FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"generator.heat_w\" " +
        "FROM ( SELECT " +
            "generator2.time AS time, generator2.heat_w AS \"generator.heat_w\" " +
            "FROM generator2  " +
            "OFFSET 0" +
        ") AS x WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for single heatpump source", () => {
    const queries = privateFunctions.mkqueries({
        generators: [],
        heatpumps: [7],
    });

    // prettier-ignore
    const expected_sql =
        "WITH heatpump7 AS (" +
            "SELECT time, power_w*cop_pct/100.0 AS heat_w, power_w, " +
                "cop_pct / 100.0 AS cop " +
            "FROM heatpumps " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, " +
            "\"heatpump.power_w\", \"heatpump.heat_w\", \"heatpump.cop\" " +
        "FROM ( SELECT " +
            "heatpump7.time AS time, " +
            "heatpump7.power_w AS \"heatpump.power_w\", " +
            "heatpump7.heat_w AS \"heatpump.heat_w\", " +
            "heatpump7.cop AS \"heatpump.cop\" " +
            "FROM heatpump7  " +
            "OFFSET 0" +
        ") AS x WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for combined source", () => {
    const queries = privateFunctions.mkqueries({
        generators: [2],
        heatpumps: [7],
    });

    // prettier-ignore
    const expected_sql =
        "WITH heatpump7 AS (" +
            "SELECT time, power_w*cop_pct/100.0 AS heat_w, power_w, " +
                "cop_pct / 100.0 AS cop " +
            "FROM heatpumps " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        "), generator2 AS (" +
            "SELECT time, power_w * (1-0.138)/0.138 * 1.11 AS heat_w " +
            "FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") SELECT time, " +
            "\"heatpump.power_w\", \"heatpump.heat_w\", \"heatpump.cop\", " +
            "\"generator.heat_w\" " +
        "FROM ( SELECT " +
            "heatpump7.time AS time, " +
            "heatpump7.power_w AS \"heatpump.power_w\", " +
            "heatpump7.heat_w AS \"heatpump.heat_w\", " +
            "heatpump7.cop AS \"heatpump.cop\", " +
            "generator2.heat_w AS \"generator.heat_w\" " +
            "FROM heatpump7 " +
            "FULL OUTER JOIN generator2 ON heatpump7.time = generator2.time " +
            "OFFSET 0" +
        ") AS x WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for combined multi source", () => {
    const queries = privateFunctions.mkqueries({
        generators: [2, 3],
        heatpumps: [7, 8],
    });

    // prettier-ignore
    const expected_sql =
        "WITH heatpump7 AS (" +
            "SELECT time, power_w*cop_pct/100.0 AS heat_w, power_w, " +
                "cop_pct / 100.0 AS cop " +
            "FROM heatpumps " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        "), heatpump8 AS (" +
            "SELECT time, power_w*cop_pct/100.0 AS heat_w, power_w, " +
                "cop_pct / 100.0 AS cop " +
            "FROM heatpumps " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        "), generator2 AS (" +
            "SELECT time, power_w * (1-0.138)/0.138 * 1.11 AS heat_w " +
            "FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        "), generator3 AS (" +
            "SELECT time, power_w * (1-0.138)/0.138 * 1.11 AS heat_w " +
            "FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        ") SELECT time, " +
            "\"heatpump.power_w\", \"heatpump.heat_w\", \"heatpump.cop\", " +
            "\"generator.heat_w\" " +
        "FROM ( SELECT " +
            "heatpump7.time AS time, " +
            "COALESCE(heatpump7.power_w, 0)+COALESCE(heatpump8.power_w, 0) " +
                "AS \"heatpump.power_w\", " +
            "COALESCE(heatpump7.heat_w, 0)+COALESCE(heatpump8.heat_w, 0) " +
                "AS \"heatpump.heat_w\", " +
            "(COALESCE(heatpump7.cop, 0)+COALESCE(heatpump8.cop, 0) ) " +
                "/ NULLIF(" +
                    "CASE WHEN heatpump7.cop > 1 THEN 1 ELSE 0 END+" +
                    "CASE WHEN heatpump8.cop > 1 THEN 1 ELSE 0 END" +
                ",0) AS \"heatpump.cop\", " +
            "COALESCE(generator2.heat_w, 0)+COALESCE(generator3.heat_w, 0) " +
                "AS \"generator.heat_w\" " +
            "FROM heatpump7 " +
            "FULL OUTER JOIN heatpump8 ON heatpump7.time = heatpump8.time " +
            "FULL OUTER JOIN generator2 ON heatpump7.time = generator2.time " +
            "FULL OUTER JOIN generator3 ON heatpump7.time = generator3.time " +
            "OFFSET 0" +
        ") AS x WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
