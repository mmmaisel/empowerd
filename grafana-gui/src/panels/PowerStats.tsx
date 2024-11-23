import { DataLink } from "@grafana/data";
import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { Generator } from "../queries/Generator";
import { Solar } from "../queries/Solar";

export type DrilldownConfig = {
    solar: DataLink[];
};

const mkscene = (
    config: BackendConfig,
    dds: DrilldownConfig
): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watth")
        .setNoValue("No Data")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            override
                .matchFieldsByQuery("Solar")
                .overrideColor({ fixedColor: Colors.yellow(0), mode: "fixed" })
                .overrideDisplayName("Solar")
                .overrideLinks(dds.solar);
            override
                .matchFieldsByQuery("Generator")
                .overrideColor({ fixedColor: Colors.red(0), mode: "fixed" })
                .overrideDisplayName("Generator");
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
        {
            refId: "Generator",
            rawSql: Generator.query_energy_sum(config.generators).sql(),
            format: "table",
        },
    ];
};

// TODO: dedup
export const PowerStats = (
    config: ConfigJson,
    links: DrilldownConfig
): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });

    return {
        query: queryRunner,
        scene: mkscene(config.backend || BackendConfigDefault, links),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
