import {
    PanelBuilders,
    SceneDataTransformer,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { DefaultValueTrafo } from "../trafos/DefaultValue";
import { EmpPanelBuilder } from "./Common";
import { Color } from "./Color";
import { Generator } from "../queries/Generator";
import { Heatpump } from "../queries/Heatpump";

export class HeatSumStats extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.stat()
            .setUnit("watth")
            .setNoValue("No Data")
            .setOption("graphMode", "none" as any)
            .setOption("textMode", "value_and_name" as any)
            .setOverrides((override: any) => {
                override
                    .matchFieldsByQuery(`heatpump.heat`)
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Heatpump Heat`);
                override
                    .matchFieldsByQuery(`heatpump.cop`)
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideUnit("none")
                    .overrideDisplayName(`Heatpump CoP`);
                override
                    .matchFieldsByQuery(`generator.heat`)
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Generator Heat`);
            })
            .build();
    }

    public queries(): any[] {
        let queries: any = [];

        queries.push({
            refId: `heatpump.heat`,
            rawSql: Heatpump.query_dheat_wh_sum(this.config.heatpumps).sql(),
            format: "table",
        });
        queries.push({
            refId: `heatpump.cop`,
            rawSql: Heatpump.query_acop_sum(this.config.heatpumps).sql(),
            format: "table",
        });
        queries.push({
            refId: `generator.heat`,
            rawSql: Generator.query_dheat_wh_sum(this.config.generators).sql(),
            format: "table",
        });

        return queries;
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
            transformations: [DefaultValueTrafo],
        }) as any;
    }
}
