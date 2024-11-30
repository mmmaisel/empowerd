import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
    SceneDataTransformer,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Color } from "./Color";
import { GroupByMonthTrafo } from "../trafos/GroupByMonth";
import { Solar } from "../queries/Solar";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.barchart()
        .setOption("xTickLabelRotation", -90)
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    return [
        {
            refId: "Solar",
            rawSql: Solar.query_energy_mon(config.solars).sql(),
            format: "table",
        },
    ];
};

// TODO: dedup
export const SolarPerMonth = (config: ConfigJson): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });
    const transformedData = new SceneDataTransformer({
        $data: queryRunner,
        transformations: [
            GroupByMonthTrafo.bind(null, "Solar", "watth", Color.yellow),
        ],
    });

    return new Panel({
        query: transformedData,
        scene: mkscene(config.backend || BackendConfigDefault),
    });
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
