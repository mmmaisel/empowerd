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
        .setTitle("Rain")
        .setMin(0.1)
        .setUnit("lengthmm")
        .setCustomFieldConfig("fillOpacity", 0)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setCustomFieldConfig("scaleDistribution", {
            log: 10,
            type: "log" as any,
        })
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName(`rain_act_mm`)
                .overrideColor({
                    fixedColor: Color.cyan(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Rain Actual`);
            override
                .matchFieldsWithName(`rain_day_mm`)
                .overrideColor({
                    fixedColor: Color.blue(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Rain Day`);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Weather.query_rain(config.weathers).sql(),
            format: "table",
        },
    ];
};

export const RainPlot = (config: ConfigJson): Panel => {
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
