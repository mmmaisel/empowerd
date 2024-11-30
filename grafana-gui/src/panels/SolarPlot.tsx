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
                    .matchFieldsWithName(`solar${solar}.power_w`)
                    .overrideColor({
                        fixedColor: Color.yellow(i).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Solar ${i + 1}`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Solar.query_power(config.solars).sql(),
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

    return new Panel({
        query: queryRunner,
        scene: mkscene(config.backend || BackendConfigDefault),
    });
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
