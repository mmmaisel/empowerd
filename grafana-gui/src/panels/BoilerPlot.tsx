import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Color } from "./Color";
import { Boiler } from "../queries/Boiler";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Boiler stats")
        .setUnit("celsius")
        .setCustomFieldConfig("fillOpacity", 0)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            let i = 0;
            for (let id of config.heatpumps) {
                override
                    .matchFieldsWithName(`boiler${id}.top`)
                    .overrideColor({
                        fixedColor: Color.red(i).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Boiler ${i + 1} Top`);
                override
                    .matchFieldsWithName(`boiler${id}.mid`)
                    .overrideColor({
                        fixedColor: Color.purple(i).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Boiler ${i + 1} Middle`);
                override
                    .matchFieldsWithName(`boiler${id}.bot`)
                    .overrideColor({
                        fixedColor: Color.blue(i).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Boiler ${i + 1} Bottom`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "A",
            rawSql: Boiler.query_temps(config.heatpumps).sql(),
            format: "table",
        },
    ];
};

// TODO: dedup
export const BoilerPlot = (config: ConfigJson): Panel => {
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
