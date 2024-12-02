import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Solar } from "../queries/Solar";

export class SolarPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle("Solar stats")
            .setUnit("watt")
            .setMin(0)
            .setMax(10000)
            .setCustomFieldConfig("fillOpacity", 10)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                let i = 0;
                for (let solar of this.config.solars) {
                    override
                        .matchFieldsWithName(`solar${solar}.power_w`)
                        .overrideColor({
                            fixedColor: Color.yellow(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(`Solar ${i + 1}`);
                    i += 1;
                }
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Solar.query_power(this.config.solars).sql(),
                format: "table",
            },
        ];
    }
}
