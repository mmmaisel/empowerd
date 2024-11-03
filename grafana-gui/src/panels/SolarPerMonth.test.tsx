import "intersection-observer";

import { privateFunctions } from "./SolarPerMonth";

test("Query for single solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1] });

    // prettier-ignore
    const expected_sql =
        "WITH samples AS (" +
            "WITH months AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 MONTH'" +
                ") AS month" +
            ") " +
            "SELECT " +
            "month + INTERVAL '12 HOUR' AS start," +
            "month + INTERVAL '1 MONTH' + INTERVAL '12 HOUR' AS next " +
            "FROM months" +
        "), solar AS (" +
            "SELECT time, energy_wh FROM simple_meters WHERE series_id = 1" +
        ") " +
        "SELECT " +
        "samples.start AS month, " +
        "solar_next.energy_wh - solar_start.energy_wh AS energy_wh " +
        "FROM samples " +
        "LEFT OUTER JOIN solar AS solar_next ON solar_next.time = samples.next " +
        "LEFT OUTER JOIN solar AS solar_start ON solar_start.time = samples.start";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1, 8] });

    // prettier-ignore
    const expected_sql =
        "WITH samples AS (" +
            "WITH months AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 MONTH'" +
                ") AS month" +
            ") " +
            "SELECT " +
            "month + INTERVAL '12 HOUR' AS start," +
            "month + INTERVAL '1 MONTH' + INTERVAL '12 HOUR' AS next " +
            "FROM months" +
        "), solar AS (" +
            "WITH solar1 AS (" +
                "SELECT time, energy_wh FROM simple_meters WHERE series_id = 1" +
            "), solar8 AS (" +
                "SELECT time, energy_wh FROM simple_meters WHERE series_id = 8" +
            ") " +
            "SELECT " +
            "solar1.time AS time, " +
            "COALESCE(solar1.energy_wh, 0)+COALESCE(solar8.energy_wh, 0) " +
                "AS energy_wh " +
            "FROM solar1 " +
            "FULL OUTER JOIN solar8 ON solar1.time = solar8.time" +
        ") " +
        "SELECT " +
        "samples.start AS month, " +
        "solar_next.energy_wh - solar_start.energy_wh AS energy_wh " +
        "FROM samples " +
        "LEFT OUTER JOIN solar AS solar_next ON solar_next.time = samples.next " +
        "LEFT OUTER JOIN solar AS solar_start ON solar_start.time = samples.start";

    expect(queries[0].rawSql).toBe(expected_sql);
});
