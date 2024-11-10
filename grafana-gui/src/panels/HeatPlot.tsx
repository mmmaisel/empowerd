import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel } from "./Common";
import { Colors } from "./Colors";
import { Fragment, Field, Timeseries } from "../queries/Query";
import { ProxyQuery, TimeProxy } from "../queries/Proxy";
import { Generator, GeneratorSeries } from "../queries/Generator";
import { Heatpump, HeatpumpSeries } from "../queries/Heatpump";

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.timeseries()
        .setTitle("Heat stats")
        .setUnit("watt")
        .setMin(0)
        .setMax(10000)
        .setCustomFieldConfig("fillOpacity", 10)
        .setCustomFieldConfig("showPoints", "always" as any)
        .setCustomFieldConfig("spanNulls", false)
        .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
        .setOverrides((override: any) => {
            override
                .matchFieldsWithName("heatpump.power_w")
                .overrideColor({ fixedColor: Colors.purple(0), mode: "fixed" })
                .overrideDisplayName("Heatpump Power");
            override
                .matchFieldsWithName("heatpump.heat_w")
                .overrideColor({ fixedColor: Colors.green(0), mode: "fixed" })
                .overrideDisplayName("Heatpump Heat");
            override
                .matchFieldsWithName("heatpump.cop")
                .overrideUnit("none")
                .overrideMax(10)
                .overrideColor({ fixedColor: Colors.yellow(0), mode: "fixed" })
                .overrideDisplayName("Heatpump CoP")
                .overrideCustomFieldConfig("fillOpacity", 0);
            override
                .matchFieldsWithName("generator.heat_w")
                .overrideColor({ fixedColor: Colors.red(0), mode: "fixed" })
                .overrideDisplayName("Generator Heat");
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let query = null;
    if (config.generators.length === 0) {
        query = Heatpump.query_all(config.heatpumps);
    } else if (config.heatpumps.length === 0) {
        query = Generator.query_heat(config.generators);
    } else {
        const heatpumps = config.heatpumps.map((id) =>
            new HeatpumpSeries(id)
                .time()
                .heat(null)
                .power(null)
                .cop(null)
                .time_filter()
        );
        const generators = config.generators.map((id) =>
            new GeneratorSeries(id).time().heat(null).time_filter()
        );

        let first = "";
        let heatpump_ids = [...config.heatpumps];
        let generator_ids = [...config.generators];
        if (heatpump_ids.length !== 0) {
            first = `heatpump${config.heatpumps[0]}`;
            heatpump_ids.shift();
        } else if (generator_ids.length !== 0) {
            first = `generator${config.generators[0]}`;
            generator_ids.shift();
        }

        const fields = [
            new TimeProxy(first),
            HeatpumpSeries.ps_heat(config.heatpumps).with_alias(
                `\"heatpump.heat_w\"`
            ),
            HeatpumpSeries.ps_power(config.heatpumps).with_alias(
                `\"heatpump.power_w\"`
            ),
            HeatpumpSeries.pa_cop(config.heatpumps).with_alias(
                `\"heatpump.cop\"`
            ),
            GeneratorSeries.ps_heat(config.generators).with_alias(
                `\"generator.heat_w\"`
            ),
        ];

        query = new Timeseries()
            .subqueries([...heatpumps, ...generators])
            .fields([
                new Field(`time`, null),
                new Field(`\"heatpump.heat_w\"`, null),
                new Field(`\"heatpump.power_w\"`, null),
                new Field(`\"heatpump.cop\"`, null),
                new Field(`\"generator.heat_w\"`, null),
            ])
            .from(
                new ProxyQuery()
                    .fields(fields)
                    .from(new Fragment(first))
                    .joins(
                        [
                            HeatpumpSeries.time_join(first, heatpump_ids),
                            GeneratorSeries.time_join(first, generator_ids),
                        ].flat()
                    )
            )
            .time_not_null()
            .ordered();
    }

    return [
        {
            refId: "A",
            rawSql: query.sql(),
            format: "table",
        },
    ];
};

// TODO: dedup
export const HeatPlot = (config: ConfigJson): Panel => {
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
