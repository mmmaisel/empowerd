import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Solar } from "../queries/Solar";
import { t } from "../i18n";

export class SolarPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        let builder = PanelBuilders.timeseries()
            .setHoverHeader(true)
            .setUnit("watt")
            .setMin(0)
            .setMax(this.config.ranges.production[1])
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
                        .overrideDisplayName(t("solar-n", { id: i + 1 }));
                    i += 1;
                }
            });

        this.build_menu(builder);
        return builder.build();
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
