import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Solar stats")
        .setUnit("watt")
        .setMin(0)
        .setMax(10000)
        .setCustomFieldConfig("fillOpacity", 10)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            let i = 0;
            for (let solar of config.solars) {
                override
                    .matchFieldsWithName(`solar${solar}.power`)
                    .overrideColor({
                        fixedColor: Colors.yellow(i),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Solar ${i + 1}`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let first_table = "";
    let sources: string[] = [];
    let selects: string[] = [];
    let columns: string[] = [];
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
        selects.push(`\"solar${solar}.power\"`);
        columns.push(`solar${solar}.power_w AS \"solar${solar}.power\"`);
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
            `${columns.join(", ")} ` +
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
export const SolarPlot = (config: ConfigJson): Panel => {
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
