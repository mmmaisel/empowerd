import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { WindPlot } from "./WindPlot";

test("Query for single weather source", () => {
    const queries = new WindPlot({
        ...BackendConfigDefault,
        weathers: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, " +
                "wind_act_mms/1000.0 AS wind_act_ms, " +
                "wind_gust_mms/1000.0 AS wind_gust_ms, " +
                "wind_dir_deg_e1/10.0 AS wind_dir_deg " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
