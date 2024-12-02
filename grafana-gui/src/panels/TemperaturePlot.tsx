import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Weather } from "../queries/Weather";

export class TemperaturePlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle("Temperature")
            .setUnit("celsius")
            .setCustomFieldConfig("fillOpacity", 0)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName(`temp_in_degc`)
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Inside`);
                override
                    .matchFieldsWithName(`temp_out_degc`)
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Outside`);
                override
                    .matchFieldsWithName(`dew_point_degc`)
                    .overrideColor({
                        fixedColor: Color.purple(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Dew Point`);
                override
                    .matchFieldsWithName(`temp_x1_degc`)
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`X1`);
                override
                    .matchFieldsWithName(`temp_x2_degc`)
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`X2`);
                override
                    .matchFieldsWithName(`temp_x3_degc`)
                    .overrideColor({
                        fixedColor: Color.orange(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`X3`);
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Weather.query_temps(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }
}
