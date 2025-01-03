import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { SolarPerMonth } from "./SolarPerMonth";

test("Query for single solar source", () => {
    const queries = new SolarPerMonth({
        ...BackendConfigDefault,
        solars: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "WITH samples AS (" +
            "WITH points AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 MONTH'" +
                ") AS point" +
            ") " +
            "SELECT " +
            "point + INTERVAL '12 HOUR' AS start, " +
            "point + INTERVAL '1 MONTH' + INTERVAL '12 HOUR' AS next " +
            "FROM points" +
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
    const queries = new SolarPerMonth({
        ...BackendConfigDefault,
        solars: [1, 8],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "WITH samples AS (" +
            "WITH points AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 MONTH'" +
                ") AS point" +
            ") " +
            "SELECT " +
            "point + INTERVAL '12 HOUR' AS start, " +
            "point + INTERVAL '1 MONTH' + INTERVAL '12 HOUR' AS next " +
            "FROM points" +
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
