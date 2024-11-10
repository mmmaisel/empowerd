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
import { Battery } from "../queries/Battery";
import { Colors } from "./Colors";
import { Heating } from "../queries/Heating";
import { Meter } from "../queries/Meter";
import { Panel } from "./Common";
import { Production } from "../queries/Production";

export type DrilldownConfig = {
    power: DataLink[];
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
                .matchFieldsByQuery("Production")
                .overrideColor({
                    fixedColor: Colors.green(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Power Production`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Meter")
                .overrideColor({
                    fixedColor: Colors.red(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Meter Power`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Battery Power")
                .overrideColor({
                    fixedColor: Colors.blue(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Battery Power`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Battery Charge")
                .overrideColor({
                    fixedColor: Colors.grey(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Battery Charge`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Heat")
                .overrideColor({
                    fixedColor: Colors.purple(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Heat Production`)
                .overrideLinks(dds.heatpump);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    queries.push({
        refId: "Production",
        rawSql: Production.query_power_sum(config).sql(),
        format: "table",
    });
    queries.push({
        refId: "Meter",
        rawSql: Meter.query_power_sum(config.meters).sql(),
        format: "table",
    });
    queries.push({
        refId: "Battery Power",
        rawSql: Battery.query_power_sum(config.batteries).sql(),
        format: "table",
    });
    queries.push({
        refId: "Battery Charge",
        rawSql: Battery.query_charge_sum(config.batteries).sql(),
        format: "table",
    });
    queries.push({
        refId: "Heat",
        rawSql: Heating.query_heat_sum(config).sql(),
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
