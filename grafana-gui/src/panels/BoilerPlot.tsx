import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel, hsl_to_rgb } from "./Common";

const colors_top = [
    hsl_to_rgb(0, 206, 110),
    hsl_to_rgb(340, 206, 110),
    hsl_to_rgb(20, 206, 110),
    hsl_to_rgb(0, 206, 150),
    hsl_to_rgb(340, 206, 150),
    hsl_to_rgb(20, 206, 150),
    hsl_to_rgb(0, 206, 90),
    hsl_to_rgb(340, 206, 90),
    hsl_to_rgb(20, 206, 90),
];

const colors_mid = [
    hsl_to_rgb(280, 130, 120),
    hsl_to_rgb(270, 130, 120),
    hsl_to_rgb(290, 130, 120),
    hsl_to_rgb(280, 130, 150),
    hsl_to_rgb(270, 130, 150),
    hsl_to_rgb(290, 130, 150),
    hsl_to_rgb(280, 130, 90),
    hsl_to_rgb(270, 130, 90),
    hsl_to_rgb(290, 130, 90),
];

const colors_bot = [
    hsl_to_rgb(220, 186, 110),
    hsl_to_rgb(210, 186, 110),
    hsl_to_rgb(230, 186, 110),
    hsl_to_rgb(220, 186, 150),
    hsl_to_rgb(210, 186, 150),
    hsl_to_rgb(230, 186, 150),
    hsl_to_rgb(220, 186, 90),
    hsl_to_rgb(210, 186, 90),
    hsl_to_rgb(230, 186, 90),
];

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Boiler stats")
        .setUnit("celsius")
        .setCustomFieldConfig("fillOpacity", 0)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            let i = 0;
            for (let id of config.heatpumps) {
                override
                    .matchFieldsWithName(`boiler${id}.top`)
                    .overrideColor({
                        fixedColor: colors_top[i % colors_top.length],
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Boiler ${i + 1} Top`);
                override
                    .matchFieldsWithName(`boiler${id}.mid`)
                    .overrideColor({
                        fixedColor: colors_mid[i % colors_mid.length],
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Boiler ${i + 1} Middel`);
                override
                    .matchFieldsWithName(`boiler${id}.bot`)
                    .overrideColor({
                        fixedColor: colors_bot[i % colors_bot.length],
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Boiler ${i + 1} Bottom`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let first_table = "";
    let sources: string[] = [];
    let selects: string[] = [];
    let joins: string[] = [];

    // TODO: handle empty config correctly

    // T out
    // prettier-ignore
    "SELECT time, temp_out_degc_e1 / 10.0 AS \"weather.temperature_out\" " +
    "FROM weathers " +
    "WHERE series_id = 5 AND $__timeFilter(time) ORDER BY time"

    for (let id of config.heatpumps) {
        if (first_table === "") {
            first_table = `boiler${id}`;
        } else {
            joins.push(`boiler${id}`);
        }

        sources.push(
            `boiler${id} AS (` +
                `SELECT time, ` +
                `boiler_top_degc_e1 / 10.0 AS top, ` +
                `boiler_mid_degc_e1 / 10.0 AS mid, ` +
                `boiler_bot_degc_e1 / 10.0 AS bot ` +
                `FROM heatpumps ` +
                `WHERE series_id = ${id} AND $__timeFilter(time)` +
                `)`
        );
        selects.push(`boiler${id}.top`);
        selects.push(`boiler${id}.mid`);
        selects.push(`boiler${id}.bot`);
    }

    let join_sql = joins.map(
        (x) => `FULL OUTER JOIN ${x} ON ${first_table}.time = ${x}.time`
    );

    const sql =
        // prettier-ignore
        `WITH ${sources.join(", ")} ` +
        `SELECT time, ${selects.map(x => `"${x}"`).join(", ")} ` +
        `FROM ( SELECT ` +
            `${first_table}.time AS time, ` +
            `${selects.map(x => `${x} AS "${x}"`).join(", ")} ` +
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
export const BoilerPlot = (config: ConfigJson): Panel => {
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
