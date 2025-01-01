import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class RainPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle(t("rain"))
            .setMin(0.1)
            .setUnit("lengthmm")
            .setCustomFieldConfig("fillOpacity", 0)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setCustomFieldConfig("scaleDistribution", {
                log: 10,
                type: "log" as any,
            })
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName(`rain_act_mm`)
                    .overrideColor({
                        fixedColor: Color.cyan(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("rain-act"));
                override
                    .matchFieldsWithName(`rain_day_mm`)
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("rain-day"));
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Weather.query_rain(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }
}
