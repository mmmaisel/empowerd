import "intersection-observer";

import { privateFunctions } from "./SolarStats";

test("Query for dual solar source", () => {
    const queries = privateFunctions.mkqueries({ solars: [1, 8] });

    // prettier-ignore
    const expected_sql = (i: number): string => {
        return `SELECT MAX(energy_wh) - MIN(energy_wh) ` +
        `AS \"solar${i}.energy\"` +
        `FROM simple_meters ` +
        `WHERE series_id = ${i} AND $__timeFilter(time)`;
    };

    expect(queries[0].rawSql).toBe(expected_sql(1));
    expect(queries[1].rawSql).toBe(expected_sql(8));
});
