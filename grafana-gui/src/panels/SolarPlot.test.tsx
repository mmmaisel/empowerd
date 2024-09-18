import "intersection-observer";

import { privateFunctions } from "./SolarPlot";

test("Query for single solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1] });

    // prettier-ignore
    const expected_sql =
        "WITH solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"solar1.power\" " +
        "FROM ( SELECT " +
            "solar1.time AS time, solar1.power_w AS \"solar1.power\" " +
            "FROM solar1  " +
            "OFFSET 0" +
        ") AS x WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1, 8] });

    // prettier-ignore
    const expected_sql =
        "WITH solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar8 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"solar1.power\", \"solar8.power\" " +
        "FROM ( SELECT " +
            "solar1.time AS time, " +
            "solar1.power_w AS \"solar1.power\", " +
            "solar8.power_w AS \"solar8.power\" " +
            "FROM solar1 " +
            "FULL OUTER JOIN solar8 ON solar1.time = solar8.time " +
            "OFFSET 0" +
        ") AS x WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
