import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Boiler } from "../queries/Boiler";

export class BoilerPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setHoverHeader(true)
            .setUnit("celsius")
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
                        .overrideDisplayName(`Boiler ${i + 1} Top`);
                    override
                        .matchFieldsWithName(`boiler${id}.mid`)
                        .overrideColor({
                            fixedColor: Color.purple(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(`Boiler ${i + 1} Middle`);
                    override
                        .matchFieldsWithName(`boiler${id}.bot`)
                        .overrideColor({
                            fixedColor: Color.blue(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(`Boiler ${i + 1} Bottom`);
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
