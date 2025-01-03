import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { BaroPlot } from "./BaroPlot";

test("Query for single weather source", () => {
    const queries = new BaroPlot({
        ...BackendConfigDefault,
        weathers: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, " +
                "baro_abs_pa/100.0 AS baro_abs_hpa, " +
                "baro_sea_pa/100.0 AS baro_sea_hpa " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
