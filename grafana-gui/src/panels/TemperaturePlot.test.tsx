import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { TemperaturePlot } from "./TemperaturePlot";

test("Query for single weather source with all ext sensors", () => {
    const queries = new TemperaturePlot({
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
            "temp_in_degc_e1/10.0 AS temp_in_degc, " +
            "temp_out_degc_e1/10.0 AS temp_out_degc, " +
            "dew_point_degc_e1/10.0 AS dew_point_degc, " +
            "temp_x1_degc_e1/10.0 AS temp_x1_degc, " +
            "temp_x2_degc_e1/10.0 AS temp_x2_degc, " +
            "temp_x3_degc_e1/10.0 AS temp_x3_degc, " +
            "temp_x4_degc_e1/10.0 AS temp_x4_degc, " +
            "temp_x5_degc_e1/10.0 AS temp_x5_degc, " +
            "temp_x6_degc_e1/10.0 AS temp_x6_degc, " +
            "temp_x7_degc_e1/10.0 AS temp_x7_degc " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for single weather source with some ext sensors", () => {
    const queries = new TemperaturePlot({
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
            "temp_in_degc_e1/10.0 AS temp_in_degc, " +
            "temp_out_degc_e1/10.0 AS temp_out_degc, " +
            "dew_point_degc_e1/10.0 AS dew_point_degc, " +
            "temp_x1_degc_e1/10.0 AS temp_x1_degc, " +
            "temp_x2_degc_e1/10.0 AS temp_x2_degc, " +
            "temp_x3_degc_e1/10.0 AS temp_x3_degc " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
