import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { BoilerPlot } from "./BoilerPlot";

test("Query for single boiler source", () => {
    const queries = new BoilerPlot({
        ...BackendConfigDefault,
        heatpumps: [1],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT time, " +
            "boiler_top_degc_e1 / 10.0 AS \"boiler1.top\", " +
            "boiler_mid_degc_e1 / 10.0 AS \"boiler1.mid\", " +
            "boiler_bot_degc_e1 / 10.0 AS \"boiler1.bot\" " +
            "FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time) " +
            "ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual boiler source", () => {
    const queries = new BoilerPlot({
        ...BackendConfigDefault,
        heatpumps: [1, 7],
    }).queries();

    // prettier-ignore
    const expected_sql =
        "WITH boiler1 AS (" +
            "SELECT time, " +
            "boiler_top_degc_e1 / 10.0 AS top, " +
            "boiler_mid_degc_e1 / 10.0 AS mid, " +
            "boiler_bot_degc_e1 / 10.0 AS bot " +
            "FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), boiler7 AS (" +
            "SELECT time, " +
            "boiler_top_degc_e1 / 10.0 AS top, " +
            "boiler_mid_degc_e1 / 10.0 AS mid, " +
            "boiler_bot_degc_e1 / 10.0 AS bot " +
            "FROM heatpumps " +
            "WHERE series_id = 7 AND $__timeFilter(time)" +
        ") " +
        "SELECT time, \"boiler1.top\", \"boiler1.mid\", \"boiler1.bot\", " +
            "\"boiler7.top\", \"boiler7.mid\", \"boiler7.bot\" " +
        "FROM (SELECT " +
            "boiler1.time AS time, " +
            "boiler1.top AS \"boiler1.top\", " +
            "boiler1.mid AS \"boiler1.mid\", " +
            "boiler1.bot AS \"boiler1.bot\", " +
            "boiler7.top AS \"boiler7.top\", " +
            "boiler7.mid AS \"boiler7.mid\", " +
            "boiler7.bot AS \"boiler7.bot\" " +
            "FROM boiler1 " +
            "FULL OUTER JOIN boiler7 ON boiler1.time = boiler7.time " +
            "OFFSET 0" +
        ") AS proxy WHERE time IS NOT NULL ORDER BY time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
