import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { GeneratorSeries } from "../queries/Generator";
import { HeatpumpSeries } from "../queries/Heatpump";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watth")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            let i = 0;
            for (let id of config.heatpumps) {
                override
                    .matchFieldsWithName(`heatpump${id}.heat_wh`)
                    .overrideColor({
                        fixedColor: Colors.green(i),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Heatpump ${i + 1} Heat`);
                override
                    .matchFieldsWithName(`heatpump${id}.cop`)
                    .overrideColor({
                        fixedColor: Colors.yellow(i),
                        mode: "fixed",
                    })
                    .overrideUnit("none")
                    .overrideDisplayName(`Heatpump ${i + 1} CoP`);
                i += 1;
            }

            i = 0;
            for (let id of config.generators) {
                override
                    .matchFieldsWithName(`generator${id}.heat_wh`)
                    .overrideColor({
                        fixedColor: Colors.red(i),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Generator ${i + 1} Heat`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    for (let id of config.heatpumps) {
        queries.push({
            refId: `heatpump${id}.heat`,
            rawSql: new HeatpumpSeries(id)
                .d_heat(`\"heatpump${id}.heat_wh\"`)
                .time_filter()
                .sql(),
            format: "table",
        });
        queries.push({
            refId: `heatpump${id}.cop`,
            rawSql: new HeatpumpSeries(id)
                .a_cop(`\"heatpump${id}.cop\"`)
                .time_filter()
                .sql(),
            format: "table",
        });
    }

    for (let id of config.generators) {
        queries.push({
            refId: `generator${id}.heat`,
            rawSql: new GeneratorSeries(id)
                .d_heat(`\"generator${id}.heat_wh\"`)
                .time_filter()
                .sql(),
            format: "table",
        });
    }

    return queries;
};

// TODO: dedup
export const HeatStats = (config: ConfigJson): Panel => {
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
