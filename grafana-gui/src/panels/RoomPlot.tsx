import {
    PanelBuilders,
    SceneObject,
    SceneObjectState,
    SceneDataTransformer,
    SceneQueryRunner,
} from "@grafana/scenes";

import { Color } from "./Color";
import { RoomDetailsTrafo } from "../trafos/RoomDetails";
import { EmpPanelBuilder } from "./Common";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export class RoomPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        let builder = PanelBuilders.timeseries()
            .setTitle(t("room", { room: "${room}" }))
            .setUnit("celsius")
            .setCustomFieldConfig("fillOpacity", 0)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName(`temp_degc`)
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("temperature"));
                override
                    .matchFieldsWithName(`humidity_pct`)
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideUnit("humidity")
                    .overrideDisplayName(t("humidity"));
                override
                    .matchFieldsWithName(`dew_point_degc`)
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("dew-point"));
                override
                    .matchFieldsWithName(`humidity_kgm3`)
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideUnit("congm3")
                    .overrideDisplayName(t("abs_humidity"));
            });

        this.build_menu(builder);
        return builder.build();
    }

    public queries(): any[] {
        return [
            {
                refId: "A",
                rawSql: Weather.query_room_var(this.config.weathers).sql(),
                format: "table",
            },
        ];
    }

    protected query_runner(): SceneQueryRunner {
        const queryRunner = new SceneQueryRunner({
            datasource: {
                uid: this.ds_uid,
            },
            queries: this.queries(),
        });

        return new SceneDataTransformer({
            $data: queryRunner,
            transformations: [RoomDetailsTrafo],
        }) as any;
    }
}
