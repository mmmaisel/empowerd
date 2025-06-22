import { PanelBuilders, SceneObject, SceneObjectState } from "@grafana/scenes";

import { Battery } from "../queries/Battery";
import { Color } from "./Color";
import { EmpPanelBuilder } from "./Common";
import { Generator } from "../queries/Generator";
import { Meter } from "../queries/Meter";
import { Solar } from "../queries/Solar";
import { t } from "../i18n";

export class PowerProductionPlot extends EmpPanelBuilder {
    public scene(): SceneObject<SceneObjectState> {
        let builder = PanelBuilders.timeseries()
            .setHoverHeader(true)
            .setUnit("watt")
            .setMin(this.config.ranges.production[0])
            .setMax(this.config.ranges.production[1])
            .setCustomFieldConfig("fillOpacity", 10)
            .setCustomFieldConfig("showPoints", "always" as any)
            .setCustomFieldConfig("spanNulls", false)
            .setCustomFieldConfig("thresholdsStyle", {
                mode: "line+area" as any,
            })
            .setThresholds({
                mode: "absolute" as any,
                steps: [{ color: "transparent", value: null as any }],
            })
            .setOption("tooltip", { mode: "multi" as any, sort: "none" as any })
            .setOverrides((override: any) => {
                override
                    .matchFieldsWithName("solar.power_w")
                    .overrideColor({
                        fixedColor: Color.yellow(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("solar-pwr"));
                override
                    .matchFieldsWithName("generator.power_w")
                    .overrideColor({
                        fixedColor: Color.green(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("generator-pwr"));
                override
                    .matchFieldsWithName("meter.power_w")
                    .overrideColor({
                        fixedColor: Color.red(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("meter-pwr"));
                override
                    .matchFieldsWithName("battery.power_w")
                    .overrideColor({
                        fixedColor: Color.blue(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideDisplayName(t("battery-pwr"));
                override
                    .matchFieldsWithName("battery.charge_wh")
                    .overrideUnit("watth")
                    .overrideMin(0)
                    .overrideMax(this.config.ranges.battery[1])
                    .overrideColor({
                        fixedColor: Color.grey(0).to_rgb(),
                        mode: "fixed",
                    })
                    .overrideThresholds({
                        mode: "absolute",
                        steps: [
                            { value: null, color: "dark-red" },
                            {
                                value: this.config.ranges.battery[0],
                                color: "transparent",
                            },
                            {
                                value: this.config.ranges.battery[1],
                                color: "dark-green",
                            },
                        ],
                    })
                    .overrideCustomFieldConfig("fillOpacity", 0)
                    .overrideDisplayName(t("battery-charge"));
            });

        this.build_menu(builder);
        return builder.build();
    }

    public queries(): any[] {
        // TODO: use one big query
        let queries: any = [];

        if (this.config.solars.length !== 0) {
            queries.push({
                refId: "A",
                rawSql: Solar.query_power_sum(this.config.solars).sql(),
                format: "table",
            });
        }
        if (this.config.generators.length !== 0) {
            queries.push({
                refId: "B",
                rawSql: Generator.query_power_sum(this.config.generators).sql(),
                format: "table",
            });
        }
        if (this.config.meters.length !== 0) {
            queries.push({
                refId: "C",
                rawSql: Meter.query_power_sum(this.config.meters).sql(),
                format: "table",
            });
        }
        if (this.config.batteries.length !== 0) {
            queries.push({
                refId: "D",
                rawSql: Battery.query_power_charge_sum(
                    this.config.batteries
                ).sql(),
                format: "table",
            });
        }

        return queries;
    }
}
