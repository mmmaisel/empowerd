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
        .setTitle("Heat stats")
        .setUnit("watt")
        .setMin(0)
        .setMax(10000)
        .setCustomFieldConfig("fillOpacity", 10)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName("heatpump.power_w")
                .overrideColor({ fixedColor: Colors.purple(0), mode: "fixed" })
                .overrideDisplayName("Heatpump Power");
            override
                .matchFieldsWithName("heatpump.heat_w")
                .overrideColor({ fixedColor: Colors.green(0), mode: "fixed" })
                .overrideDisplayName("Heatpump Heat");
            override
                .matchFieldsWithName("heatpump.cop")
                .overrideUnit("none")
                .overrideMax(10)
                .overrideColor({ fixedColor: Colors.yellow(0), mode: "fixed" })
                .overrideDisplayName("Heatpump CoP")
                .overrideCustomFieldConfig("fillOpacity", 0);
            override
                .matchFieldsWithName("generator.heat_w")
                .overrideColor({ fixedColor: Colors.red(0), mode: "fixed" })
                .overrideDisplayName("Generator Heat");
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let first_table = "";
    let sources: string[] = [];
    let selects: string[] = [];
    let hp_rows: string[] = [];
    let gen_rows: string[] = [];
    let rows: string[] = [];
    let joins: string[] = [];

    // TODO: handle empty config correctly

    for (let id of config.heatpumps) {
        if (first_table === "") {
            first_table = `heatpump${id}`;
        } else {
            joins.push(`heatpump${id}`);
        }

        sources.push(
            `heatpump${id} AS (` +
                `SELECT time, power_w*cop_pct/100.0 AS heat_w, power_w, ` +
                `cop_pct / 100.0 AS cop ` +
                `FROM heatpumps ` +
                `WHERE series_id = ${id} AND $__timeFilter(time)` +
                `)`
        );
        hp_rows.push(`heatpump${id}`);
    }

    for (let id of config.generators) {
        if (first_table === "") {
            first_table = `generator${id}`;
        } else {
            joins.push(`generator${id}`);
        }

        sources.push(
            `generator${id} AS (` +
                `SELECT time, power_w * (1-0.138)/0.138 * 1.11 AS heat_w ` +
                `FROM generators ` +
                `WHERE series_id = ${id} AND $__timeFilter(time)` +
                `)`
        );
        gen_rows.push(`generator${id}`);
    }

    if (config.heatpumps.length !== 0) {
        selects.push('"heatpump.power_w"');
        selects.push('"heatpump.heat_w"');
        selects.push('"heatpump.cop"');
        if (config.heatpumps.length === 1) {
            rows.push(`${hp_rows[0]}.power_w AS \"heatpump.power_w\"`);
            rows.push(`${hp_rows[0]}.heat_w AS \"heatpump.heat_w\"`);
            rows.push(`${hp_rows[0]}.cop AS \"heatpump.cop\"`);
        } else {
            rows.push(
                `${hp_rows
                    .map((x: string) => `COALESCE(${x}.power_w, 0)`)
                    .join("+")} AS \"heatpump.power_w\"`
            );
            rows.push(
                `${hp_rows
                    .map((x: string) => `COALESCE(${x}.heat_w, 0)`)
                    .join("+")} AS \"heatpump.heat_w\"`
            );
            rows.push(
                `(${hp_rows
                    .map((x: string) => `COALESCE(${x}.cop, 0)`)
                    .join("+")} ) / ` +
                    `NULLIF(${hp_rows
                        .map(
                            (x: string) =>
                                `CASE WHEN ${x}.cop > 1 THEN 1 ELSE 0 END`
                        )
                        .join("+")},0) AS \"heatpump.cop\"`
            );
        }
    }
    if (config.generators.length !== 0) {
        selects.push('"generator.heat_w"');
        if (config.generators.length === 1) {
            rows.push(`${gen_rows[0]}.heat_w AS \"generator.heat_w\"`);
        } else {
            rows.push(
                `${gen_rows
                    .map((x: string) => `COALESCE(${x}.heat_w, 0)`)
                    .join("+")} AS \"generator.heat_w\"`
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
export const HeatPlot = (config: ConfigJson): Panel => {
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
