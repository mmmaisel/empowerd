import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { SolarPlot } from "./SolarPlot";

test("Query for single solar source", () => {
    const queries = new SolarPlot({
        ...BackendConfigDefault,
        solars: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, power_w AS \"solar1.power_w\" FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual solar source", () => {
    const queries = new SolarPlot({
        ...BackendConfigDefault,
        solars: [1, 8],
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
        "SELECT time, \"solar1.power_w\", \"solar8.power_w\" " +
        "FROM (SELECT " +
            "solar1.time AS time, " +
            "solar1.power_w AS \"solar1.power_w\", " +
            "solar8.power_w AS \"solar8.power_w\" " +
            "FROM solar1 " +
            "FULL OUTER JOIN solar8 ON solar1.time = solar8.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
