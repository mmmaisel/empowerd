import "intersection-observer";

import { BackendConfigDefault } from "../AppConfig";
import { SolarStats } from "./SolarStats";

test("Query for dual solar source", () => {
    const queries = new SolarStats({
        ...BackendConfigDefault,
        solars: [1, 8],
    }).queries();

    // prettier-ignore
    const expected_sql = (i: number): string => {
        return `SELECT MAX(energy_wh)-MIN(energy_wh) ` +
        `AS \"solar${i}.energy_wh\" ` +
        `FROM simple_meters ` +
        `WHERE series_id = ${i} AND $__timeFilter(time)`;
    };

    expect(queries[0].rawSql).toBe(expected_sql(1));
    expect(queries[1].rawSql).toBe(expected_sql(8));
});
