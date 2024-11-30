import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Color } from "./Color";
import { Solar } from "../queries/Solar";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watth")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            let i = 0;
            for (let solar of config.solars) {
                override
                    .matchFieldsWithName(`solar${solar}.energy_wh`)
                    .overrideColor({
                        fixedColor: Color.yellow(i).to_rgb(),
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

    for (let id of config.solars) {
        queries.push({
            refId: `solar${id}`,
            rawSql: Solar.query_energy(id).sql(),
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
