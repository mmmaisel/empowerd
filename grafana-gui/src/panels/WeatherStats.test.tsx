import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { WeatherStats } from "./WeatherStats";

test("Query total rin in interval", () => {
    const queries = new WeatherStats({
        ...BackendConfigDefault,
        weathers: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "WITH samples AS (" +
            "WITH points AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('DAY', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('DAY', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 DAY'" +
                ") AS point" +
            ") " +
            "SELECT " +
            "point + INTERVAL '23:00' AS start " +
            "FROM points" +
        "), weather AS (" +
            "SELECT time, rain_day_um/1000.0 AS rain_day_mm " +
            "FROM weathers WHERE series_id = 1" +
        ") " +
        "SELECT " +
        "SUM(weather.rain_day_mm) AS rain_int_mm " +
        "FROM samples " +
        "LEFT OUTER JOIN weather ON weather.time = samples.start";

    expect(queries[0].rawSql).toBe(expected_sql);
});
