import {
    PanelBuilders,
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";
import { DataLink } from "@grafana/data";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Battery } from "../queries/Battery";
import { Colors } from "./Colors";
import { Consumption } from "../queries/Consumption";
import { DefaultValueTrafo } from "../trafos/DefaultValue";
import { Heating } from "../queries/Heating";
import { Panel } from "./Common";
import { Production } from "../queries/Production";
import { Weather } from "../queries/Weather";

export type DrilldownConfig = {
    power: DataLink[];
    heatpump: DataLink[];
    weather: DataLink[];
};

const mkscene = (
    config: BackendConfig,
    dds: DrilldownConfig
): SceneObject<SceneObjectState> => {
    let panel = PanelBuilders.stat()
        .setUnit("watt")
        .setNoValue("No Data")
        .setOption("graphMode", "area" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOption("justifyMode", "center" as any)
        .setOverrides((override: any) => {
            override
                .matchFieldsByQuery("Production")
                .overrideColor({
                    fixedColor: Colors.green(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Power Production`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Consumption")
                .overrideColor({
                    fixedColor: Colors.red(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Power Consumption`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Battery")
                .overrideColor({
                    fixedColor: Colors.grey(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Battery`)
                .overrideLinks(dds.power);
            override
                .matchFieldsWithName("battery.power_w")
                .overrideColor({
                    fixedColor: Colors.grey(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Battery Power`)
                .overrideLinks(dds.power);
            override
                .matchFieldsWithName("battery.charge_wh")
                .overrideColor({
                    fixedColor: Colors.grey(0),
                    mode: "fixed",
                })
                .overrideUnit("watth")
                .overrideDisplayName(`Battery Charge`)
                .overrideLinks(dds.power);
            override
                .matchFieldsByQuery("Heat")
                .overrideColor({
                    fixedColor: Colors.purple(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Heat Production`)
                .overrideLinks(dds.heatpump);
            override
                .matchFieldsByQuery("Weather")
                .overrideColor({
                    fixedColor: Colors.blue(0),
                    mode: "fixed",
                })
                .overrideDisplayName(`Weather`)
                .overrideLinks(dds.weather);
            override
                .matchFieldsWithName("temp_out_degc")
                .overrideColor({
                    fixedColor: Colors.yellow(0),
                    mode: "fixed",
                })
                .overrideUnit("celsius")
                .overrideDisplayName(`Out Temperature`)
                .overrideLinks(dds.weather);
            override
                .matchFieldsWithName("rain_act_um")
                .overrideColor({
                    fixedColor: Colors.blue(0),
                    mode: "fixed",
                })
                .overrideUnit("um")
                .overrideDisplayName(`Rain`)
                .overrideLinks(dds.weather);
        });

    // Not exposed through builder interface
    (panel as any)._fieldConfigBuilder.setFieldConfigDefaults(
        "fieldMinMax",
        true
    );

    return panel.build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    queries.push({
        refId: "Production",
        rawSql: Production.query_power_sum(config).sql(),
        format: "table",
    });
    queries.push({
        refId: "Consumption",
        rawSql: Consumption.query_power_sum(config).sql(),
        format: "table",
    });
    queries.push({
        refId: "Battery",
        rawSql: Battery.query_power_charge_sum(config.batteries).sql(),
        format: "table",
    });
    queries.push({
        refId: "Heat",
        rawSql: Heating.query_heat_sum(config).sql(),
        format: "table",
    });
    queries.push({
        refId: "Weather",
        rawSql: Weather.query_temp_rain(config.weathers).sql(),
        format: "table",
    });

    return queries;
};

// TODO: dedup
export const Overview = (config: ConfigJson, links: DrilldownConfig): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });
    const transformedData = new SceneDataTransformer({
        $data: queryRunner,
        transformations: [DefaultValueTrafo],
    });

    return {
        query: transformedData,
        scene: mkscene(config.backend || BackendConfigDefault, links),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
