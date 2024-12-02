import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Generator } from "../queries/Generator";
import { Solar } from "../queries/Solar";

export class PowerPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle("Power stats")
            .setUnit("watt")
            .setMin(-8000)
            .setMax(10000)
            .setCustomFieldConfig("fillOpacity", 10)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName("solar.power_w")
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Solar");
                override
                    .matchFieldsWithName("generator.power_w")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Generator");
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Solar.query_power_sum(this.config.solars).sql(),
                format: "table",
            },
            {
                refId: "B",
                rawSql: Generator.query_power_sum(this.config.generators).sql(),
                format: "table",
            },

        ];
    }
}
