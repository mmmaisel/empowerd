import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class HumidityPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle(t("humidity"))
            .setUnit("humidity")
            .setCustomFieldConfig("fillOpacity", 0)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName(`hum_in_pct`)
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("hum-in"));
                override
                    .matchFieldsWithName(`hum_out_pct`)
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("hum-out"));
                override
                    .matchFieldsWithName(`hum_x1_pct`)
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x1);
                override
                    .matchFieldsWithName(`hum_x2_pct`)
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x2);
                override
                    .matchFieldsWithName(`hum_x3_pct`)
                    .overrideColor({
                        fixedColor: Color.orange(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x3);
                override
                    .matchFieldsWithName(`hum_x4_pct`)
                    .overrideColor({
                        fixedColor: Color.cyan(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x4);
                override
                    .matchFieldsWithName(`hum_x5_pct`)
                    .overrideColor({
                        fixedColor: Color.grey(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x5);
                override
                    .matchFieldsWithName(`hum_x6_pct`)
                    .overrideColor({
                        fixedColor: Color.green(4).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(this.config.labels.x6);
                override
                    .matchFieldsWithName(`hum_x7_pct`)
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
                rawSql: Weather.query_hums(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }
}
