import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { Consumption } from "../queries/Consumption";
import { EmpPanelBuilder } from "./Common";
import { Heatpump } from "../queries/Heatpump";
import { Wallbox } from "../queries/Wallbox";

export class PowerConsumptionPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setHoverHeader(true)
            .setUnit("watt")
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
                    .overrideDisplayName("Heatpump Power");
                override
                    .matchFieldsWithName("wallbox.power_w")
                    .overrideColor({
                        fixedColor: Color.orange(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Wallbox Power");
                override
                    .matchFieldsWithName("consumption.power_w")
                    .overrideColor({
                        fixedColor: Color.cyan(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Other Consumption");
            })
            .build();
    }

    public queries(): any[] {
        // TODO: use one big query
        let queries: any = [];

        if (this.config.heatpumps.length !== 0) {
            queries.push({
                refId: "A",
                rawSql: Heatpump.query_power_sum(this.config.heatpumps).sql(),
                format: "table",
            });
        }
        if (this.config.wallboxes.length !== 0) {
            queries.push({
                refId: "B",
                rawSql: Wallbox.query_power_sum(this.config.wallboxes).sql(),
                format: "table",
            });
        }
        if (this.config.meters.length !== 0) {
            queries.push({
                refId: "C",
                rawSql: Consumption.query_power_sum(this.config).sql(),
                format: "table",
            });
        }

        return queries;
    }
}
