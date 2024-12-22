import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
    SceneDataTransformer,
} from "@grafana/scenes";

import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { GroupByMonthTrafo } from "../trafos/GroupByMonth";
import { Solar } from "../queries/Solar";

export class SolarPerMonth extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.barchart()
            .setHoverHeader(true)
            .setOption("xTickLabelRotation", -90)
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "Solar",
                rawSql: Solar.query_energy_mon(this.config.solars).sql(),
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
            transformations: [
                GroupByMonthTrafo.bind(null, "Solar", "watth", Color.yellow),
            ],
        }) as any;
    }
}
