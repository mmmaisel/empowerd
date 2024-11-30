import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Color } from "./Color";
import { Weather } from "../queries/Weather";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Humidity")
        .setUnit("humidity")
        .setCustomFieldConfig("fillOpacity", 0)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName(`hum_in_pct`)
                .overrideColor({
                    fixedColor: Color.yellow(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Inside`);
            override
                .matchFieldsWithName(`hum_out_pct`)
                .overrideColor({
                    fixedColor: Color.blue(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Outside`);
            override
                .matchFieldsWithName(`hum_x1_pct`)
                .overrideColor({
                    fixedColor: Color.red(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`X1`);
            override
                .matchFieldsWithName(`hum_x2_pct`)
                .overrideColor({
                    fixedColor: Color.green(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`X2`);
            override
                .matchFieldsWithName(`hum_x3_pct`)
                .overrideColor({
                    fixedColor: Color.orange(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`X3`);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Weather.query_hums(config.weathers).sql(),
            format: "table",
        },
    ];
};

export const HumidityPlot = (config: ConfigJson): Panel => {
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
