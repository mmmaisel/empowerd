import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { RoomPlot } from "./RoomPlot";

test("Query for single weather source", () => {
    const queries = new RoomPlot({
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
            "temp_${room}_degc_e1/10.0 AS temp_${room}_degc, " +
            "hum_${room}_e3/10.0 AS hum_${room}_pct " +
            "FROM weathers " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
