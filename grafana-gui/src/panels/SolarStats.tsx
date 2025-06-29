import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Solar } from "../queries/Solar";
import { t } from "../i18n";

export class SolarStats extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        let builder = PanelBuilders.stat()
            .setHoverHeader(true)
            .setUnit("watth")
            .setNoValue(t("no-data"))
            .setOption("graphMode", "none" as any)
            .setOption("textMode", "value_and_name" as any)
            .setOverrides((override: any) => {
                let i = 0;
                for (let solar of this.config.solars) {
                    override
                        .matchFieldsWithName(`solar${solar}.energy_wh`)
                        .overrideColor({
                            fixedColor: Color.yellow(i).to_rgb(),
                            mode: "fixed",
                        })
                        .overrideDisplayName(
                            t("solar-n-energy", { id: i + 1 })
                        );
                    i += 1;
                }
            });

        this.build_menu(builder);
        return builder.build();
    }

    public queries(): any[] {
        let queries: any = [];

        for (let id of this.config.solars) {
            queries.push({
                refId: `solar${id}`,
                rawSql: Solar.query_denergy(id).sql(),
                format: "table",
            });
        }

        return queries;
    }
}
