import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { GeneratorSeries } from "../queries/Generator";
import { HeatpumpSeries } from "../queries/Heatpump";

export class HeatStats extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.stat()
            .setUnit("watth")
            .setOption("graphMode", "none" as any)
            .setOption("textMode", "value_and_name" as any)
            .setOverrides((override: any) => {
                let i = 0;
                for (let id of this.config.heatpumps) {
                    override
                        .matchFieldsWithName(`heatpump${id}.heat_wh`)
                        .overrideColor({
                            fixedColor: Color.green(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(`Heatpump ${i + 1} Heat`);
                    override
                        .matchFieldsWithName(`heatpump${id}.cop`)
                        .overrideColor({
                            fixedColor: Color.yellow(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideUnit("none")
                        .overrideDisplayName(`Heatpump ${i + 1} CoP`);
                    i += 1;
                }

                i = 0;
                for (let id of this.config.generators) {
                    override
                        .matchFieldsWithName(`generator${id}.heat_wh`)
                        .overrideColor({
                            fixedColor: Color.red(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(`Generator ${i + 1} Heat`);
                    i += 1;
                }
            })
            .build();
    }

    public queries(): any[] {
        let queries: any = [];

        for (let id of this.config.heatpumps) {
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

        for (let id of this.config.generators) {
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
    }
}
