import "intersection-observer";

import { privateFunctions } from "./RainPlot";

test("Query for single weather source", () => {
    const queries = privateFunctions.mkqueries({ weathers: [1] });

    // prettier-ignore
    const expected_sql =
        "SELECT time, " +
                "rain_act_um/1000.0 AS rain_act_mm, " +
                "rain_day_um/1000.0 AS rain_day_mm " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
