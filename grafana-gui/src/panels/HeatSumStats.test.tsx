import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { HeatSumStats } from "./HeatSumStats";

test("Query for single heatpump and generator source", () => {
    const queries = new HeatSumStats({
        ...BackendConfigDefault,
        heatpumps: [1],
        generators: [2],
    }).queries();

    // prettier-ignore
    const hp_query =
        "SELECT MAX(heat_wh)-MIN(heat_wh) AS d_heat_wh " +
        "FROM heatpumps " +
        "WHERE series_id = 1 AND $__timeFilter(time)";

    // prettier-ignore
    const cop_query =
        "SELECT AVG(cop_pct) / 100.0 AS \"heatpump.cop\" " +
        "FROM heatpumps " +
        "WHERE series_id = 1 AND cop_pct > 100 AND $__timeFilter(time)";

    // prettier-ignore
    const gen_query =
        "SELECT (MAX(runtime_s)-MIN(runtime_s)) * 18.48928 " +
        "AS \"generator.heat_wh\" " +
        "FROM generators " +
        "WHERE series_id = 2 AND $__timeFilter(time)";

    expect(queries[0].rawSql).toBe(hp_query);
    expect(queries[1].rawSql).toBe(cop_query);
    expect(queries[2].rawSql).toBe(gen_query);
});

test("Query for dual heatpump and generator source", () => {
    const queries = new HeatSumStats({
        ...BackendConfigDefault,
        heatpumps: [1, 2],
        generators: [3, 4],
    }).queries();

    // prettier-ignore
    const hp_query =
        "WITH heatpump1 AS (" +
            "SELECT time, heat_wh FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), heatpump2 AS (" +
            "SELECT time, heat_wh FROM heatpumps " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT COALESCE(MAX(heatpump1.heat_wh)-MIN(heatpump1.heat_wh), 0)+" +
            "COALESCE(MAX(heatpump2.heat_wh)-MIN(heatpump2.heat_wh), 0) " +
            "AS \"heatpump.heat_wh\" " +
        "FROM heatpump1 " +
        "FULL OUTER JOIN heatpump2 ON heatpump1.time = heatpump2.time";

    // prettier-ignore
    const cop_query =
        "WITH heatpump1 AS (" +
            "SELECT time, cop_pct / 100.0 AS cop FROM heatpumps " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), heatpump2 AS (" +
            "SELECT time, cop_pct / 100.0 AS cop FROM heatpumps " +
            "WHERE series_id = 2 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "(COALESCE(heatpump1.cop, 0)+COALESCE(heatpump2.cop, 0)) " +
                "/ NULLIF(" +
                    "CASE WHEN heatpump1.cop > 1 THEN 1 ELSE 0 END+" +
                    "CASE WHEN heatpump2.cop > 1 THEN 1 ELSE 0 END" +
                ", 0) AS \"heatpump.cop\" " +
        "FROM heatpump1 " +
        "FULL OUTER JOIN heatpump2 ON heatpump1.time = heatpump2.time";

    // prettier-ignore
    const gen_query =
        "WITH generator3 AS (" +
            "SELECT time, energy_wh * 6.93348 AS heat_wh FROM generators " +
            "WHERE series_id = 3 AND $__timeFilter(time)" +
        "), generator4 AS (" +
            "SELECT time, energy_wh * 6.93348 AS heat_wh FROM generators " +
            "WHERE series_id = 4 AND $__timeFilter(time)" +
        ") " +
        "SELECT COALESCE(MAX(generator3.heat_wh)-MIN(generator3.heat_wh), 0)+" +
            "COALESCE(MAX(generator4.heat_wh)-MIN(generator4.heat_wh), 0) " +
            "AS \"generator.heat_wh\" " +
        "FROM generator3 " +
        "FULL OUTER JOIN generator4 ON generator3.time = generator4.time";

    expect(queries[0].rawSql).toBe(hp_query);
    expect(queries[1].rawSql).toBe(cop_query);
    expect(queries[2].rawSql).toBe(gen_query);
});
