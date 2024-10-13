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
    return PanelBuilders.stat()
        .setUnit("watth")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            override
                .matchFieldsByQuery("Solar")
                .overrideColor({ fixedColor: Colors.yellow(0), mode: "fixed" })
                .overrideDisplayName("Solar");
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let first_table = "";
    let sources: string[] = [];
    let solar_froms: string[] = [];
    let deltas: string[] = [];
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
                `SELECT time, energy_wh FROM simple_meters ` +
                `WHERE series_id = ${solar} AND $__timeFilter(time)` +
                `)`
        );
        solar_froms.push(`solar${solar}.energy_wh`);
    }

    if (config.solars.length !== 0) {
        if (solar_froms.length === 1) {
            deltas.push(`MAX(${solar_froms[0]})-MIN(${solar_froms[0]})`);
        } else {
            deltas = solar_froms.map(
                (x: string) => `COALESCE(MAX(${x}), 0)-COALESCE(MIN(${x}), 0)`
            );
        }
    }

    let join_sql = joins.map(
        (x) => `FULL OUTER JOIN ${x} ON ${first_table}.time = ${x}.time`
    );

    const sql =
        // prettier-ignore
        `WITH ${sources.join(", ")} ` +
        `SELECT ` +
            `${deltas.join("+")} ` +
            `FROM ${first_table} ` +
            `${join_sql.join(" ")}`;

    return [
        {
            refId: "Solar",
            rawSql: sql,
            format: "table",
        },
    ];
};

// TODO: dedup
export const PowerStats = (config: ConfigJson): Panel => {
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
