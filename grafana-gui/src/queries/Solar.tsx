import { QueryBuilder, QueryFragment } from "./Builder";

type Column = {
    id: number;
    name: string;
};

export class SolarQuery {
    _solars: number[];
    _sum: boolean;

    constructor() {
        this._solars = [];
        this._sum = false;
    }

    solars = (solars: number[]): SolarQuery => {
        this._solars = solars;
        return this;
    };

    sum = (): SolarQuery => {
        this._sum = true;
        return this;
    };

    query(): QueryFragment {
        let first_table = "";
        let sources: string[] = [];
        let selects: string[] = [];
        let solar_columns: Column[] = [];
        let columns: string[] = [];
        let solar_joins: string[] = [];

        for (let solar of this._solars) {
            if (first_table === "") {
                first_table = `solar${solar}`;
            } else {
                solar_joins.push(`solar${solar}`);
            }

            sources.push(
                `solar${solar} AS (` +
                    `SELECT time, power_w FROM simple_meters ` +
                    `WHERE series_id = ${solar} AND $__timeFilter(time)` +
                    `)`
            );
            solar_columns.push({ id: solar, name: `solar${solar}.power_w` });
        }

        if (this._solars.length !== 0) {
            if (solar_columns.length === 1) {
                if (this._sum) {
                    selects.push('"solar.power"');
                    columns.push(`${solar_columns[0].name} AS \"solar.power\"`);
                } else {
                    selects.push(`\"solar${solar_columns[0].id}.power\"`);
                    columns.push(
                        `${solar_columns[0].name} AS \"solar${solar_columns[0].id}.power\"`
                    );
                }
            } else {
                if (this._sum) {
                    selects.push('"solar.power"');
                    columns.push(
                        `${solar_columns
                            .map((x: Column) => `COALESCE(${x.name}, 0)`)
                            .join("+")} ` + `AS \"solar.power\"`
                    );
                } else {
                    for (let col of solar_columns) {
                        selects.push(`\"solar${col.id}.power\"`);
                        columns.push(`${col.name} AS \"solar${col.id}.power\"`);
                    }
                }
            }
        }

        let joins = solar_joins.map(
            (x) => `FULL OUTER JOIN ${x} ON ${first_table}.time = ${x}.time`
        );

        return {
            sources,
            selects,
            table: first_table,
            columns,
            joins,
        };
    }

    timeseries = (): string => {
        return QueryBuilder.timeseries([this.query()]);
    };
}
