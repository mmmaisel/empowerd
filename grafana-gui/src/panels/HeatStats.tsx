import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { GeneratorSeries } from "../queries/Generator";
import { HeatpumpSeries } from "../queries/Heatpump";
import { t } from "../i18n";

export class HeatStats extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        let builder = PanelBuilders.stat()
            .setHoverHeader(true)
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
                        .overrideDisplayName(
                            t("heatpump-n-heat", { id: i + 1 })
                        );
                    override
                        .matchFieldsWithName(`heatpump${id}.cop`)
                        .overrideColor({
                            fixedColor: Color.yellow(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideUnit("none")
                        .overrideDisplayName(
                            t("heatpump-n-cop", { id: i + 1 })
                        );
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
                        .overrideDisplayName(
                            t("generator-n-cop", { id: i + 1 })
                        );
                    i += 1;
                }
            });

        this.build_menu(builder);
        return builder.build();
    }

    public queries(): any[] {
        let queries: any = [];

        for (let id of this.config.heatpumps) {
            queries.push({
                refId: `heatpump${id}.heat`,
                rawSql: new HeatpumpSeries(id)
                    .d_heat_wh(`\"heatpump${id}.heat_wh\"`)
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
                    .d_heat_wh(`\"generator${id}.heat_wh\"`)
                    .time_filter()
                    .sql(),
                format: "table",
            });
        }

        return queries;
    }
}
