import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Boiler } from "../queries/Boiler";
import { t } from "../i18n";

export class BoilerPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setHoverHeader(true)
            .setUnit("celsius")
            .setMin(this.config.ranges.boiler[0])
            .setMax(this.config.ranges.boiler[1])
            .setCustomFieldConfig("fillOpacity", 0)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                let i = 0;
                for (let id of this.config.heatpumps) {
                    override
                        .matchFieldsWithName(`boiler${id}.top`)
                        .overrideColor({
                            fixedColor: Color.red(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(t("boiler-n-top", { id: i + 1 }));
                    override
                        .matchFieldsWithName(`boiler${id}.mid`)
                        .overrideColor({
                            fixedColor: Color.purple(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(t("boiler-n-mid", { id: i + 1 }));
                    override
                        .matchFieldsWithName(`boiler${id}.bot`)
                        .overrideColor({
                            fixedColor: Color.blue(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(t("boiler-n-bot", { id: i + 1 }));
                    i += 1;
                }
            })
            .build();
    }

    public queries(): any[] {
        if (this.config.heatpumps.length !== 0) {
            return [
                {
                    refId: "A",
                    rawSql: Boiler.query_temps(this.config.heatpumps).sql(),
                    format: "table",
                },
            ];
        } else {
            return [];
        }
    }
}
