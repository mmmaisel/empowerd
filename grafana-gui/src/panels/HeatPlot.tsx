import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Fragment, Field, Timeseries } from "../queries/Query";
import { ProxyQuery, TimeProxy } from "../queries/Proxy";
import { Generator, GeneratorSeries } from "../queries/Generator";
import { Heatpump, HeatpumpSeries } from "../queries/Heatpump";

export class HeatPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
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
                    .overrideColor({
                        fixedColor: Color.purple(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Heatpump Power")
                    .overrideCustomFieldConfig("fillOpacity", 0);
                override
                    .matchFieldsWithName("heatpump.heat_w")
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Heatpump Heat");
                override
                    .matchFieldsWithName("heatpump.cop")
                    .overrideUnit("none")
                    .overrideMax(10)
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Heatpump CoP")
                    .overrideCustomFieldConfig("fillOpacity", 0);
                override
                    .matchFieldsWithName("generator.heat_w")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Generator Heat");
            })
            .build();
    }

    public queries(): any[] {
        let query = null;
        if (this.config.generators.length === 0) {
            query = Heatpump.query_all(this.config.heatpumps);
        } else if (this.config.heatpumps.length === 0) {
            query = Generator.query_heat(this.config.generators);
        } else {
            const heatpumps = this.config.heatpumps.map((id) =>
                new HeatpumpSeries(id)
                    .time()
                    .heat(null)
                    .power(null)
                    .cop(null)
                    .time_filter()
            );
            const generators = this.config.generators.map((id) =>
                new GeneratorSeries(id).time().heat(null).time_filter()
            );

            let first = "";
            let heatpump_ids = [...this.config.heatpumps];
            let generator_ids = [...this.config.generators];
            if (heatpump_ids.length !== 0) {
                first = `heatpump${this.config.heatpumps[0]}`;
                heatpump_ids.shift();
            } else if (generator_ids.length !== 0) {
                first = `generator${this.config.generators[0]}`;
                generator_ids.shift();
            }

            const fields = [
                TimeProxy.from_series([...heatpumps, ...generators]),
                HeatpumpSeries.ps_heat(this.config.heatpumps).with_alias(
                    `\"heatpump.heat_w\"`
                ),
                HeatpumpSeries.ps_power(this.config.heatpumps).with_alias(
                    `\"heatpump.power_w\"`
                ),
                HeatpumpSeries.pa_cop(this.config.heatpumps).with_alias(
                    `\"heatpump.cop\"`
                ),
                GeneratorSeries.ps_heat(this.config.generators).with_alias(
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
    }
}
