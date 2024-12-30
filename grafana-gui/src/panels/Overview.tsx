import {
    PanelBuilders,
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";
import { DataLink } from "@grafana/data";
import { DataSourceRef } from "@grafana/schema";

import { BackendConfig } from "../AppConfig";
import { Battery } from "../queries/Battery";
import { Color } from "./Color";
import { Consumption } from "../queries/Consumption";
import { DefaultValueTrafo } from "../trafos/DefaultValue";
import { EmpPanelBuilder } from "./Common";
import { Heating } from "../queries/Heating";
import { Production } from "../queries/Production";
import { t } from "../i18n";
import { Weather } from "../queries/Weather";

export type DrilldownConfig = {
    power: DataLink[];
    heatpump: DataLink[];
    weather: DataLink[];
};

export class Overview extends EmpPanelBuilder {
    private dds: DrilldownConfig;

    constructor(
        config: BackendConfig | undefined,
        datasource: DataSourceRef | undefined,
        dds: DrilldownConfig
    ) {
        super(config, datasource);
        this.dds = dds;
    }

    public scene(): SceneObject<SceneObjectState> {
        let panel = PanelBuilders.stat()
            // TODO: setMenu: VizPanelMenu to add zoom menu
            .setHoverHeader(true)
            .setUnit("watt")
            .setNoValue(t("no-data"))
            .setOption("graphMode", "area" as any)
            .setOption("textMode", "value_and_name" as any)
            .setOption("justifyMode", "center" as any)
            .setOverrides((override: any) => {
                override
                    .matchFieldsByQuery("Production")
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("pwr-prod"))
                    .overrideLinks(this.dds.power);
                override
                    .matchFieldsByQuery("Consumption")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("pwr-cons"))
                    .overrideLinks(this.dds.power);
                override
                    .matchFieldsByQuery("Battery")
                    .overrideColor({
                        fixedColor: Color.grey(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("battery"))
                    .overrideLinks(this.dds.power);
                override
                    .matchFieldsWithName("battery.power_w")
                    .overrideColor({
                        fixedColor: Color.grey(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("battery-pwr"))
                    .overrideLinks(this.dds.power);
                override
                    .matchFieldsWithName("battery.charge_wh")
                    .overrideColor({
                        fixedColor: Color.grey(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideUnit("watth")
                    .overrideDisplayName(t("battery-charge"))
                    .overrideLinks(this.dds.power);
                override
                    .matchFieldsByQuery("Heat")
                    .overrideColor({
                        fixedColor: Color.purple(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("heat-prod"))
                    .overrideLinks(this.dds.heatpump);
                override
                    .matchFieldsByQuery("Weather")
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("weather"))
                    .overrideLinks(this.dds.weather);
                override
                    .matchFieldsWithName("temp_out_degc")
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideUnit("celsius")
                    .overrideDisplayName(t("out-temp"))
                    .overrideLinks(this.dds.weather);
                override
                    .matchFieldsWithName("rain_act_mm")
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideUnit("lengthmm")
                    .overrideDisplayName(t("rain"))
                    .overrideLinks(this.dds.weather);
            });

        // Not exposed through builder interface
        (panel as any)._fieldConfigBuilder.setFieldConfigDefaults(
            "fieldMinMax",
            true
        );

        return panel.build();
    }

    public queries(): any[] {
        let queries: any = [];

        if (
            this.config.solars.length !== 0 ||
            this.config.generators.length !== 0
        ) {
            queries.push({
                refId: "Production",
                rawSql: Production.query_power_sum(this.config).sql(),
                format: "table",
            });
        }
        // TODO: live production/consumption is wrong because of missing latest point from solar
        // TODO: a wallbox without anything else does not work correctly
        if (this.config.meters.length !== 0) {
            queries.push({
                refId: "Consumption",
                rawSql: Consumption.query_power_sum({
                    batteries: this.config.batteries,
                    generators: this.config.generators,
                    heatpumps: [],
                    meters: this.config.meters,
                    solars: this.config.solars,
                    wallboxes: [],
                }).sql(),
                format: "table",
            });
        }
        if (this.config.batteries.length !== 0) {
            queries.push({
                refId: "Battery",
                rawSql: Battery.query_power_charge_sum(
                    this.config.batteries
                ).sql(),
                format: "table",
            });
        }
        if (
            this.config.generators.length !== 0 ||
            this.config.heatpumps.length !== 0
        ) {
            queries.push({
                refId: "Heat",
                rawSql: Heating.query_heat_sum(this.config).sql(),
                format: "table",
            });
        }
        if (this.config.weathers.length !== 0) {
            queries.push({
                refId: "Weather",
                rawSql: Weather.query_temp_rain(this.config.weathers).sql(),
                format: "table",
            });
        }

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
