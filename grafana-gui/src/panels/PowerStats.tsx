import { DataLink } from "@grafana/data";
import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { BackendConfig } from "../AppConfig";
import { EmpPanelBuilder } from "./Common";
import { Color } from "./Color";
import { Battery } from "../queries/Battery";
import { Consumption } from "../queries/Consumption";
import { Generator } from "../queries/Generator";
import { Meter } from "../queries/Meter";
import { Solar } from "../queries/Solar";
import { Wallbox } from "../queries/Wallbox";

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
            .setOption("wideLayout", true)
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
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Generator");
                override
                    .matchFieldsWithName("battery.d_energy_in_wh")
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Battery Charged");
                override
                    .matchFieldsWithName("battery.d_energy_out_wh")
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Battery Discharged");
                override
                    .matchFieldsWithName("meter.d_energy_in_wh")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Meter In");
                override
                    .matchFieldsWithName("meter.d_energy_out_wh")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Meter Out");
                override
                    .matchFieldsByQuery("Wallbox")
                    .overrideColor({
                        fixedColor: Color.orange(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Wallbox");
                override
                    .matchFieldsByQuery("Consumption")
                    .overrideColor({
                        fixedColor: Color.cyan(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName("Other Consumption");
            })
            .build();
    }

    public queries(): any[] {
        return [
            {
                refId: "Solar",
                rawSql: Solar.query_denergy_sum(this.config.solars).sql(),
                format: "table",
            },
            {
                refId: "Generator",
                rawSql: Generator.query_energy_sum(
                    this.config.generators
                ).sql(),
                format: "table",
            },
            {
                refId: "Battery",
                rawSql: Battery.query_energy_in_out_sum(
                    this.config.batteries
                ).sql(),
                format: "table",
            },
            {
                refId: "Meter",
                rawSql: Meter.query_energy_in_out_sum(this.config.meters).sql(),
                format: "table",
            },
            {
                refId: "Wallbox",
                rawSql: Wallbox.query_denergy_sum(this.config.wallboxes).sql(),
                format: "table",
            },
            {
                refId: "Consumption",
                rawSql: Consumption.query_denergy_sum(this.config).sql(),
                format: "table",
            },
        ];
    }
}
