import "intersection-observer";

import { privateFunctions } from "./HumidityPlot";

test("Query for single weather source", () => {
    const queries = privateFunctions.mkqueries({ weathers: [1] });

    // prettier-ignore
    const expected_sql =
        "SELECT time, " +
            "hum_in_e3/10.0 AS hum_in_pct, " +
            "hum_out_e3/10.0 AS hum_out_pct, " +
            "hum_x1_e3/10.0 AS hum_x1_pct, " +
            "hum_x2_e3/10.0 AS hum_x2_pct, " +
            "hum_x3_e3/10.0 AS hum_x3_pct " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
