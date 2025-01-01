import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class BaroPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.timeseries()
            .setTitle(t("barometer"))
            .setUnit("pressurehpa")
            .setCustomFieldConfig("fillOpacity", 0)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName(`baro_abs_hpa`)
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("baro-abs"));
                override
                    .matchFieldsWithName(`baro_sea_hpa`)
                    .overrideColor({
                        fixedColor: Color.green(3).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("baro-sea"));
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Weather.query_baro(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }
}
