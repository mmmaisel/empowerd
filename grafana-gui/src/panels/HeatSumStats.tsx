import {
    PanelBuilders,
    SceneDataTransformer,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { DefaultValueTrafo } from "../trafos/DefaultValue";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { Generator } from "../queries/Generator";
import { Heatpump } from "../queries/Heatpump";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watth")
        .setNoValue("No Data")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            override
                .matchFieldsByQuery(`heatpump.heat`)
                .overrideColor({
                    fixedColor: Colors.green(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Heatpump Heat`);
            override
                .matchFieldsByQuery(`heatpump.cop`)
                .overrideColor({
                    fixedColor: Colors.yellow(0),
                    mode: "fixed",
                })
                .overrideUnit("none")
                .overrideDisplayName(`Heatpump CoP`);
            override
                .matchFieldsByQuery(`generator.heat`)
                .overrideColor({
                    fixedColor: Colors.red(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Generator Heat`);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    queries.push({
        refId: `heatpump.heat`,
        rawSql: Heatpump.query_dheat_sum(config.heatpumps).sql(),
        format: "table",
    });
    queries.push({
        refId: `heatpump.cop`,
        rawSql: Heatpump.query_acop_sum(config.heatpumps).sql(),
        format: "table",
    });
    queries.push({
        refId: `generator.heat`,
        rawSql: Generator.query_dheat_sum(config.generators).sql(),
        format: "table",
    });

    return queries;
};

// TODO: dedup
export const HeatSumStats = (config: ConfigJson): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });
    const transformedData = new SceneDataTransformer({
        $data: queryRunner,
        transformations: [DefaultValueTrafo],
    });

    return {
        query: transformedData,
        scene: mkscene(config.backend || BackendConfigDefault),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
