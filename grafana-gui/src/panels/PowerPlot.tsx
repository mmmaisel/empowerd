import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { Solar } from "../queries/Solar";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Power stats")
        .setUnit("watt")
        .setMin(-8000)
        .setMax(10000)
        .setCustomFieldConfig("fillOpacity", 10)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName("solar.power_w")
                .overrideColor({ fixedColor: Colors.yellow(0), mode: "fixed" })
                .overrideDisplayName("Solar");
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Solar.query_power_sum(config.solars).sql(),
            format: "table",
        },
    ];
};

// TODO: dedup
export const PowerPlot = (config: ConfigJson): Panel => {
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
