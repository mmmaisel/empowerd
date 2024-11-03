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
    return [
        {
            refId: "Solar",
            rawSql: Solar.query_energy_sum(config.solars).sql(),
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
