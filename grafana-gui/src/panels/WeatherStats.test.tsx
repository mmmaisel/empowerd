import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { WeatherStats } from "./WeatherStats";

test("Query total rain in interval", () => {
    const queries = new WeatherStats({
        ...BackendConfigDefault,
        weathers: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT (MAX(rain_acc_um)-MIN(rain_acc_um))/1000.0 " +
        "AS \"weather1.rain_acc_mm\" " +
        "FROM weathers " +
        "WHERE series_id = 1 AND $__timeFilter(time)";

    expect(queries[0].rawSql).toBe(expected_sql);
});
