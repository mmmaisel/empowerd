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
        .setTitle("Barometer")
        .setUnit("pressurehpa")
        .setCustomFieldConfig("fillOpacity", 0)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName(`baro_abs_hpa`)
                .overrideColor({
                    fixedColor: Color.green(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Absolute Pressure`);
            override
                .matchFieldsWithName(`baro_sea_hpa`)
                .overrideColor({
                    fixedColor: Color.green(3).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Sea Level Pressure`);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Weather.query_baro(config.weathers).sql(),
            format: "table",
        },
    ];
};

export const BaroPlot = (config: ConfigJson): Panel => {
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
