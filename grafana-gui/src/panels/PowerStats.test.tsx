import "intersection-observer";

import { privateFunctions } from "./PowerStats";

test("Query for single solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1] });

    // prettier-ignore
    const expected_sql =
        "WITH solar1 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        ") " +
        "SELECT MAX(solar1.energy_wh)-MIN(solar1.energy_wh) " +
        "FROM solar1 ";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1, 8] });

    // prettier-ignore
    const expected_sql =
        "WITH solar1 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar8 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(MAX(solar1.energy_wh), 0)" +
            "-COALESCE(MIN(solar1.energy_wh), 0)" +
            "+COALESCE(MAX(solar8.energy_wh), 0)" +
            "-COALESCE(MIN(solar8.energy_wh), 0) " +
        "FROM solar1 " +
        "FULL OUTER JOIN solar8 ON solar1.time = solar8.time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
