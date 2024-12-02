import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { PowerStats } from "./PowerStats";

test("Query for single solar source", () => {
    const queries = new PowerStats(
        { ...BackendConfigDefault, solars: [1], generators: [] },
        undefined,
        undefined as any
    ).queries();

    // prettier-ignore
    const expected_sql =
        "SELECT MAX(energy_wh)-MIN(energy_wh) AS \"solar.energy_wh\" " +
        "FROM simple_meters " +
        "WHERE series_id = 1 AND $__timeFilter(time)";

    expect(queries[0].rawSql).toBe(expected_sql);
});

test("Query for dual solar source", () => {
    const queries = new PowerStats(
        {
            ...BackendConfigDefault,
            solars: [1, 8],
            generators: [],
        },
        undefined,
        undefined as any
    ).queries();

    // prettier-ignore
    const expected_sql =
        "WITH solar1 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 1 AND $__timeFilter(time)" +
        "), solar8 AS (" +
            "SELECT time, energy_wh FROM simple_meters " +
            "WHERE series_id = 8 AND $__timeFilter(time)" +
        ") " +
        "SELECT " +
            "COALESCE(MAX(solar1.energy_wh), 0)" +
            "-COALESCE(MIN(solar1.energy_wh), 0)" +
            "+COALESCE(MAX(solar8.energy_wh), 0)" +
            "-COALESCE(MIN(solar8.energy_wh), 0) AS \"solar.energy_wh\" " +
        "FROM solar1 " +
        "FULL OUTER JOIN solar8 ON solar1.time = solar8.time";

    expect(queries[0].rawSql).toBe(expected_sql);
});
