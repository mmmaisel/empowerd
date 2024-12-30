import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class WeatherStats extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.stat()
            .setHoverHeader(true)
            .setUnit("lengthmm")
            .setNoValue(t("no-data"))
            .setOption("graphMode", "none" as any)
            .setOption("textMode", "value_and_name" as any)
            .setOverrides((override: any) => {
                override
                    .matchFieldsByQuery("rain_int")
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("rain-interval"));
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: `rain_int`,
                rawSql: Weather.query_rain_int(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }
}
