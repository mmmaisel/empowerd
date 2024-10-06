import "intersection-observer";

import { privateFunctions } from "./HeatStats";

test("Query for heatpump and generator source", () => {
    const queries = privateFunctions.mkqueries({
        heatpumps: [1],
        generators: [2],
    });

    // prettier-ignore
    const hp_query =
        "SELECT MAX(heat_wh)-MIN(heat_wh) AS \"heatpump1.heat\" " +
        "FROM heatpumps " +
        "WHERE series_id = 1 AND $__timeFilter(time)";

    // prettier-ignore
    const cop_query =
        "SELECT AVG(cop_pct)/100.0 AS \"heatpump1.cop\" " +
        "FROM heatpumps " +
        "WHERE series_id = 1 AND $__timeFilter(time) " +
        "AND cop_pct > 100";

    // prettier-ignore
    const gen_query =
        "SELECT (MAX(runtime_s)-MIN(runtime_s)) * 18.48928 " +
        "AS \"generator2.heat\" " +
        "FROM generators " +
        "WHERE series_id = 2 AND $__timeFilter(time)";

    expect(queries[0].rawSql).toBe(hp_query);
    expect(queries[1].rawSql).toBe(cop_query);
    expect(queries[2].rawSql).toBe(gen_query);
});
