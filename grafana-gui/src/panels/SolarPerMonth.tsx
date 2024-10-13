import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
    SceneDataTransformer,
} from "@grafana/scenes";
import { DataFrame } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";

const month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const groupByMonthsTransformation =
    () =>
    (source: Observable<DataFrame[]>): Observable<DataFrame[]> => {
        return source.pipe(
            map((data: DataFrame[]) => {
                if (data.length === 0) {
                    return [];
                }

                let av_months: Set<number> = new Set();
                let av_years: Set<number> = new Set();
                for (let timestamp of data[0].fields[0].values) {
                    let date = new Date(timestamp);
                    av_months.add(date.getMonth());
                    av_years.add(date.getFullYear());
                }

                const first_month = [...av_months][0];
                const first_year = Math.min(...av_years);
                let months = {
                    name: "month",
                    type: "string" as any,
                    config: {},
                    values: [...av_months].map((x: number) => month_names[x]),
                };
                let years: any[] = [...av_years].map((x: number) => {
                    return {
                        name: `Solar ${x}`,
                        type: "number" as any,
                        config: {
                            color: {
                                fixedColor: Colors.yellow(x - first_year),
                                mode: "fixed",
                            },
                            unit: "watth",
                        },
                        values: Array(12).fill(null),
                    };
                });

                for (let i = 0; i < data[0].fields[0].values.length; ++i) {
                    let date = new Date(data[0].fields[0].values[i]);
                    years[date.getFullYear() - first_year].values[
                        (date.getMonth() - first_month + 12) % 12
                    ] = data[0].fields[1].values[i];
                }

                return [
                    {
                        fields: [months, ...years],
                        length: years.length + 1,
                        refId: "A",
                    },
                ];
            })
        );
    };

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.barchart()
        .setOption("xTickLabelRotation", -90)
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let first_table = "";
    let sources: string[] = [];
    let solar_rows: string[] = [];
    let sum = "";
    let joins: string[] = [];

    // TODO: handle empty config correctly

    for (let solar of config.solars) {
        if (first_table === "") {
            first_table = `solar${solar}`;
        } else {
            joins.push(`solar${solar}`);
        }

        sources.push(
            // prettier-ignore
            `solar${solar} AS (` +
                `SELECT time, energy_wh FROM simple_meters ` +
                `WHERE series_id = ${solar}` +
            `)`
        );
        solar_rows.push(`solar${solar}.energy_wh`);
    }

    if (config.solars.length !== 0) {
        sum = solar_rows.map((x: string) => `COALESCE(${x}, 0)`).join("+");
    }

    let join_sql = joins.map(
        (x) => `FULL OUTER JOIN ${x} ON ${first_table}.time = ${x}.time`
    );

    const sql =
        // prettier-ignore
        "WITH samples AS (" +
            "WITH months AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 MONTH'" +
                ") AS month" +
            ") " +
            "SELECT " +
            "month + INTERVAL '12 HOUR' AS start," +
            "month + INTERVAL '1 MONTH' + INTERVAL '12 HOUR' AS next " +
            "FROM months" +
        "), solar AS (" +
            `WITH ${sources.join(", ")}` +
            "SELECT " +
            `${first_table}.time AS time, ${sum} AS energy_wh ` +
            `FROM ${first_table} ` +
            `${join_sql.join(" ")}` +
        ") " +
        "SELECT " +
        "samples.start AS month," +
        "solar_next.energy_wh - solar_start.energy_wh AS energy_wh " +
        "FROM samples " +
        "LEFT OUTER JOIN solar AS solar_next ON solar_next.time = samples.next " +
        "LEFT OUTER JOIN solar AS solar_start ON solar_start.time = samples.start";

    return [
        {
            refId: "Solar",
            rawSql: sql,
            format: "table",
        },
    ];
};

// TODO: dedup
export const SolarPerMonth = (config: ConfigJson): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });
    const transformedData = new SceneDataTransformer({
        $data: queryRunner,
        transformations: [groupByMonthsTransformation],
    });

    return {
        query: transformedData,
        scene: mkscene(config.backend || BackendConfigDefault),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
