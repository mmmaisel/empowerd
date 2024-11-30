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
        .setTitle("Wind")
        .setMin(0.1)
        .setUnit("velocityms")
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
                .matchFieldsWithName(`wind_act_ms`)
                .overrideColor({
                    fixedColor: Color.orange(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Average Wind Speed`);
            override
                .matchFieldsWithName(`wind_gust_ms`)
                .overrideColor({
                    fixedColor: Color.red(0).to_rgb(),
                    mode: "fixed",
                })
                .overrideDisplayName(`Gust Wind Speed`);
            override
                .matchFieldsWithName(`wind_dir_deg`)
                .overrideMin(0.0)
                .overrideMax(360.0)
                .overrideUnit("deg")
                .overrideColor({
                    fixedColor: Color.yellow(0).with_alpha(0.2).to_rgba(),
                    mode: "fixed",
                })
                .overrideCustomFieldConfig("scaleDistribution", {
                    type: "linear" as any,
                })
                .overrideDisplayName(`Wind Direction`);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Weather.query_wind(config.weathers).sql(),
            format: "table",
        },
    ];
};

export const WindPlot = (config: ConfigJson): Panel => {
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
