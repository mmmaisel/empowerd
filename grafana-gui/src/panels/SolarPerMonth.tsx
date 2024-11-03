import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
    SceneDataTransformer,
} from "@grafana/scenes";
import { DataFrame } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { Solar } from "../queries/Solar";

const month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const groupByMonthsTransformation =
    () =>
    (source: Observable<DataFrame[]>): Observable<DataFrame[]> => {
        return source.pipe(
            map((data: DataFrame[]) => {
                if (data.length === 0) {
                    return [];
                }

                let av_months: Set<number> = new Set();
                let av_years: Set<number> = new Set();
                for (let timestamp of data[0].fields[0].values) {
                    let date = new Date(timestamp);
                    av_months.add(date.getMonth());
                    av_years.add(date.getFullYear());
                }

                const first_month = [...av_months][0];
                const first_year = Math.min(...av_years);
                let months = {
                    name: "month",
                    type: "string" as any,
                    config: {},
                    values: [...av_months].map((x: number) => month_names[x]),
                };
                let years: any[] = [...av_years].map((x: number) => {
                    return {
                        name: `Solar ${x}`,
                        type: "number" as any,
                        config: {
                            color: {
                                fixedColor: Colors.yellow(x - first_year),
                                mode: "fixed",
                            },
                            unit: "watth",
                        },
                        values: Array(12).fill(null),
                    };
                });

                for (let i = 0; i < data[0].fields[0].values.length; ++i) {
                    let date = new Date(data[0].fields[0].values[i]);
                    years[date.getFullYear() - first_year].values[
                        (date.getMonth() - first_month + 12) % 12
                    ] = data[0].fields[1].values[i];
                }

                return [
                    {
                        fields: [months, ...years],
                        length: years.length + 1,
                        refId: "A",
                    },
                ];
            })
        );
    };

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
        transformations: [groupByMonthsTransformation],
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
