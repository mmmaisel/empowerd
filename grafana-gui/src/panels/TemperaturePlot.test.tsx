import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { TemperaturePlot } from "./TemperaturePlot";

test("Query for single weather source", () => {
    const queries = new TemperaturePlot({
        ...BackendConfigDefault,
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
