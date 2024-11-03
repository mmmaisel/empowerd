import "intersection-observer";

import { privateFunctions } from "./Overview";

test("Query for single sources", () => {
    const queries = privateFunctions.mkqueries({
        solars: [1],
        generators: [2],
    });

    // prettier-ignore
    const expected_sql0 =
        "SELECT time, power_w AS \"solar.power_w\" FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    // prettier-ignore
    const expected_sql1 =
        "SELECT time, power_w AS \"generator.power_w\" FROM generators " +
            "WHERE series_id = 2 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql0);
    expect(queries[1].rawSql).toBe(expected_sql1);
});

test("Query for dual", () => {
    const queries = privateFunctions.mkqueries({
        solars: [1, 2],
        generators: [3, 4],
    });

    // prettier-ignore
    const expected_sql0 =
        "WITH solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar2 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"solar.power_w\" " +
        "FROM (SELECT " +
            "solar1.time AS time, " +
            "COALESCE(solar1.power_w, 0)+COALESCE(solar2.power_w, 0) " +
                "AS \"solar.power_w\" " +
            "FROM solar1 " +
            "FULL OUTER JOIN solar2 ON solar1.time = solar2.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    // prettier-ignore
    const expected_sql1 =
        "WITH generator3 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, power_w FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"generator.power_w\" " +
        "FROM (SELECT " +
            "generator3.time AS time, " +
            "COALESCE(generator3.power_w, 0)+COALESCE(generator4.power_w, 0) " +
                "AS \"generator.power_w\" " +
            "FROM generator3 " +
            "FULL OUTER JOIN generator4 ON generator3.time = generator4.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql0);
    expect(queries[1].rawSql).toBe(expected_sql1);
});
