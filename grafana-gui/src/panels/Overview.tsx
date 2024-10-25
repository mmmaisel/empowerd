import {
    PanelBuilders,
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";
import { DataFrame } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { SolarQuery } from "../queries/Solar";

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

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
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
                // TODO: parametrize
                .overrideLinks([
                    {
                        title: "Drill down",
                        url: "${__url.path}/power",
                    },
                ]);
            override
                .matchFieldsByQuery("Foobar")
                .overrideColor({
                    fixedColor: Colors.red(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Dummy`);
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    queries.push({
        refId: "Solar",
        rawSql: new SolarQuery().solars(config.solars).sum().timeseries(),
        format: "table",
    });
    queries.push({
        refId: "Foobar",
        rawSql: "SELECT power_w FROM generators WHERE series_id = 2 AND $__timeFilter(time)",
        format: "table",
    });

    return queries;
};

// TODO: dedup
export const Overview = (config: ConfigJson): Panel => {
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
        scene: mkscene(config.backend || BackendConfigDefault),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
