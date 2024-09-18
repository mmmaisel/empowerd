import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel, hsl_to_rgb } from "./Common";

const colors = [
    hsl_to_rgb(50, 230, 128),
    hsl_to_rgb(40, 230, 128),
    hsl_to_rgb(60, 230, 128),
    hsl_to_rgb(50, 230, 160),
    hsl_to_rgb(40, 230, 160),
    hsl_to_rgb(60, 230, 160),
    hsl_to_rgb(50, 230, 96),
    hsl_to_rgb(40, 230, 96),
    hsl_to_rgb(60, 230, 96),
];

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watth")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            let i = 0;
            for (let solar of config.solars) {
                override
                    .matchFieldsWithName(`solar${solar}.energy`)
                    .overrideColor({
                        fixedColor: colors[i % colors.length],
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Solar ${i + 1} Energy`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    for (let solar of config.solars) {
        queries.push({
            refId: `solar${solar}`,
            rawSql:
                `SELECT MAX(energy_wh) - MIN(energy_wh) ` +
                `AS \"solar${solar}.energy\"` +
                `FROM simple_meters ` +
                `WHERE series_id = ${solar} AND $__timeFilter(time)`,
            format: "table",
        });
    }

    return queries;
};

// TODO: dedup
export const SolarStats = (config: ConfigJson): Panel => {
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
