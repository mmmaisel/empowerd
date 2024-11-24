import "intersection-observer";

import { privateFunctions } from "./TemperaturePlot";

test("Query for single weather source", () => {
    const queries = privateFunctions.mkqueries({ weathers: [1] });

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
