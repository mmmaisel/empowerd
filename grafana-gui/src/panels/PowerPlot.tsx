import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Power stats")
        .setUnit("watt")
        .setMin(-8000)
        .setMax(10000)
        .setCustomFieldConfig("fillOpacity", 10)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName("solar.power")
                .overrideColor({ fixedColor: "#F2CC0C", mode: "fixed" })
                .overrideDisplayName("Solar");
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let first_table = "";
    let sources: string[] = [];
    let selects: string[] = [];
    let solar_rows: string[] = [];
    let rows: string[] = [];
    let joins: string[] = [];

    // TODO: handle empty config correctly

    for (let solar of config.solars) {
        if (first_table === "") {
            first_table = `solar${solar}`;
        } else {
            joins.push(`solar${solar}`);
        }

        sources.push(
            `solar${solar} AS (` +
                `SELECT time, power_w FROM simple_meters ` +
                `WHERE series_id = ${solar} AND $__timeFilter(time)` +
                `)`
        );
        solar_rows.push(`solar${solar}.power_w`);
    }

    if (config.solars.length !== 0) {
        selects.push('"solar.power"');
        if (solar_rows.length === 1) {
            rows.push(`${solar_rows[0]} AS \"solar.power\"`);
        } else {
            rows.push(
                `${solar_rows
                    .map((x: string) => `COALESCE(${x}, 0)`)
                    .join("+")} ` + `AS \"solar.power\"`
            );
        }
    }

    let join_sql = joins.map(
        (x) => `FULL OUTER JOIN ${x} ON ${first_table}.time = ${x}.time`
    );

    const sql =
        // prettier-ignore
        `WITH ${sources.join(", ")} ` +
        `SELECT time, ${selects.join(", ")} ` +
        `FROM ( SELECT ` +
            `${first_table}.time AS time, ` +
            `${rows.join(", ")} ` +
            `FROM ${first_table} ` +
            `${join_sql.join(" ")} ` +
            `OFFSET 0` +
        `) AS x WHERE time IS NOT NULL ORDER BY time`;

    return [
        {
            refId: "A",
            rawSql: sql,
            format: "table",
        },
    ];
};

// TODO: dedup
export const PowerPlot = (config: ConfigJson): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });

    return {
        query: queryRunner,
        scene: mkscene(config.backend || BackendConfigDefault),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
