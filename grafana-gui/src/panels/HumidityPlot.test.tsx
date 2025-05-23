import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { HumidityPlot } from "./HumidityPlot";

test("Query for single weather source with all ext sensors", () => {
    const queries = new HumidityPlot({
        ...BackendConfigDefault,
        labels: {
            x1: "x1",
            x2: "x2",
            x3: "x3",
            x4: "x4",
            x5: "x5",
            x6: "x6",
            x7: "x7",
        },
        weathers: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, " +
            "hum_in_e3/10.0 AS hum_in_pct, " +
            "hum_out_e3/10.0 AS hum_out_pct, " +
            "hum_x1_e3/10.0 AS hum_x1_pct, " +
            "hum_x2_e3/10.0 AS hum_x2_pct, " +
            "hum_x3_e3/10.0 AS hum_x3_pct, " +
            "hum_x4_e3/10.0 AS hum_x4_pct, " +
            "hum_x5_e3/10.0 AS hum_x5_pct, " +
            "hum_x6_e3/10.0 AS hum_x6_pct, " +
            "hum_x7_e3/10.0 AS hum_x7_pct " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
test("Query for single weather source with some ext sensors", () => {
    const queries = new HumidityPlot({
        ...BackendConfigDefault,
        labels: {
            x1: "x1",
            x2: "x2",
            x3: "x3",
            x4: null,
            x5: null,
            x6: null,
            x7: null,
        },
        weathers: [1],
    }).queries();

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
