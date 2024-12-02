import { DataLink } from "@grafana/data";
import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { BackendConfig } from "../AppConfig";
import { EmpPanelBuilder } from "./Common";
import { Color } from "./Color";
import { Generator } from "../queries/Generator";
import { Solar } from "../queries/Solar";

export type DrilldownConfig = {
    solar: DataLink[];
};

export class PowerStats extends EmpPanelBuilder {
    private dds: DrilldownConfig;

    constructor(
        config: BackendConfig | undefined,
        datasource: any,
        dds: DrilldownConfig
    ) {
        super(config, datasource);
        this.dds = dds;
    }

    public scene(): SceneObject<SceneObjectState> {
        return PanelBuilders.stat()
            .setUnit("watth")
            .setNoValue("No Data")
            .setOption("graphMode", "none" as any)
            .setOption("textMode", "value_and_name" as any)
            .setOverrides((override: any) => {
                override
                    .matchFieldsByQuery("Solar")
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Solar")
                    .overrideLinks(this.dds.solar);
                override
                    .matchFieldsByQuery("Generator")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Generator");
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "Solar",
                rawSql: Solar.query_energy_sum(this.config.solars).sql(),
                format: "table",
            },
            {
                refId: "Generator",
                rawSql: Generator.query_energy_sum(
                    this.config.generators
                ).sql(),
                format: "table",
            },
        ];
    }
}
