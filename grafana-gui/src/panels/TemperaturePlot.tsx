import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class TemperaturePlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle(t("temperature"))
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
                    .overrideDisplayName(t("temp-in"));
                override
                    .matchFieldsWithName(`temp_out_degc`)
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("temp-out"));
                override
                    .matchFieldsWithName(`dew_point_degc`)
                    .overrideColor({
                        fixedColor: Color.purple(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("dew-point"));
                override
                    .matchFieldsWithName(`temp_x1_degc`)
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x1);
                override
                    .matchFieldsWithName(`temp_x2_degc`)
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x2);
                override
                    .matchFieldsWithName(`temp_x3_degc`)
                    .overrideColor({
                        fixedColor: Color.orange(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x3);
                override
                    .matchFieldsWithName(`temp_x4_degc`)
                    .overrideColor({
                        fixedColor: Color.cyan(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x4);
                override
                    .matchFieldsWithName(`temp_x5_degc`)
                    .overrideColor({
                        fixedColor: Color.grey(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x5);
                override
                    .matchFieldsWithName(`temp_x6_degc`)
                    .overrideColor({
                        fixedColor: Color.green(4).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x6);
                override
                    .matchFieldsWithName(`temp_x7_degc`)
                    .overrideColor({
                        fixedColor: Color.red(4).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x7);
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Weather.query_temps(
                    this.config.weathers,
                    this.config.labels
                ).sql(),
                format: "table",
            },
        ];
    }
}
