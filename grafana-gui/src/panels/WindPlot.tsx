import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class WindPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        let builder = PanelBuilders.timeseries()
            .setTitle(t("wind"))
            .setMin(0.1)
            .setUnit("velocityms")
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
                    .matchFieldsWithName(`wind_act_ms`)
                    .overrideColor({
                        fixedColor: Color.orange(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("wind-avg"));
                override
                    .matchFieldsWithName(`wind_gust_ms`)
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("wind-gust"));
                override
                    .matchFieldsWithName(`wind_dir_deg`)
                    .overrideMin(0.0)
                    .overrideMax(360.0)
                    .overrideUnit("deg")
                    .overrideColor({
                        fixedColor: Color.yellow(0).with_alpha(0.2).to_rgba(),
                        mode: "fixed",
                    })
                    .overrideCustomFieldConfig("scaleDistribution", {
                        type: "linear" as any,
                    })
                    .overrideDisplayName(t("wind-dir"));
            });

        this.build_menu(builder);
        return builder.build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Weather.query_wind(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }
}
