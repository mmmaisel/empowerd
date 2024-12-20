import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Solar } from "../queries/Solar";

export class SolarStats extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.stat()
            .setUnit("watth")
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
                        .overrideDisplayName(`Solar ${i + 1} Energy`);
                    i += 1;
                }
            })
            .build();
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
