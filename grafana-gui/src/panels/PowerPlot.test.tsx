import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { PowerPlot } from "./PowerPlot";

test("Query for single solar source", () => {
    const queries = new PowerPlot({
        ...BackendConfigDefault,
        solars: [1],
        generators: [],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, power_w AS \"solar.power_w\" FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual solar source", () => {
    const queries = new PowerPlot({
        ...BackendConfigDefault,
        solars: [1, 8],
        generators: [],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "WITH solar1 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar8 AS (" +
            "SELECT time, power_w FROM simple_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"solar.power_w\" " +
        "FROM (SELECT " +
            "solar1.time AS time, " +
            "COALESCE(solar1.power_w, 0)+COALESCE(solar8.power_w, 0) " +
                "AS \"solar.power_w\" " +
            "FROM solar1 " +
            "FULL OUTER JOIN solar8 ON solar1.time = solar8.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
