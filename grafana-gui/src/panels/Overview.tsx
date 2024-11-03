import {
    PanelBuilders,
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";
import { DataFrame, DataLink } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { Generator } from "../queries/Generator";
import { Heatpump } from "../queries/Heatpump";
import { Solar } from "../queries/Solar";

export type DrilldownConfig = {
    solar: DataLink[];
    generator: DataLink[];
    heatpump: DataLink[];
};

const defaultValueTransformation =
    () =>
    (source: Observable<DataFrame[]>): Observable<DataFrame[]> => {
        return source.pipe(
            map((data: DataFrame[]) => {
                for (let frame of data) {
                    if (frame.length === 0) {
                        frame.length = 1;
                        frame.fields = [
                            {
                                config: {},
                                name: "time",
                                type: "time" as any,
                                values: [null],
                            },
                            {
                                config: {},
                                name: "value",
                                type: "number" as any,
                                values: [null],
                            },
                        ];
                    }
                }
                return data;
            })
        );
    };

const mkscene = (
    config: BackendConfig,
    dds: DrilldownConfig
): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watt")
        .setNoValue("No Data")
        .setOption("graphMode", "area" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOption("justifyMode", "center" as any)
        .setOverrides((override: any) => {
            override
                .matchFieldsByQuery("Solar")
                .overrideColor({
                    fixedColor: Colors.yellow(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Solar Power`)
                .overrideLinks(dds.solar);
            override
                .matchFieldsByQuery("Generator")
                .overrideColor({
                    fixedColor: Colors.red(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Generator Power`)
                .overrideLinks(dds.generator);
            override
                .matchFieldsByQuery("Heatpump")
                .overrideColor({
                    fixedColor: Colors.purple(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Heatpump Thermal-Power`)
                .overrideLinks(dds.heatpump);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    queries.push({
        refId: "Solar",
        rawSql: Solar.query_power_sum(config.solars).sql(),
        format: "table",
    });
    queries.push({
        refId: "Generator",
        rawSql: Generator.query_power_sum(config.generators).sql(),
        format: "table",
    });
    queries.push({
        refId: "Heatpump",
        rawSql: Heatpump.query_heat_sum(config.heatpumps).sql(),
        format: "table",
    });

    return queries;
};

// TODO: dedup
export const Overview = (config: ConfigJson, links: DrilldownConfig): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });
    const transformedData = new SceneDataTransformer({
        $data: queryRunner,
        transformations: [defaultValueTransformation],
    });

    return {
        query: transformedData,
        scene: mkscene(config.backend || BackendConfigDefault, links),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
