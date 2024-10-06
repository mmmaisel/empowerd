import {
    PanelBuilders,
    SceneQueryRunner,
    SceneObject,
    SceneObjectState,
} from "@grafana/scenes";

import { BackendConfig, BackendConfigDefault, ConfigJson } from "../AppConfig";
import { Panel, hsl_to_rgb } from "./Common";

const generator_colors = [
    hsl_to_rgb(0, 206, 110),
    hsl_to_rgb(340, 206, 110),
    hsl_to_rgb(20, 206, 110),
    hsl_to_rgb(0, 206, 150),
    hsl_to_rgb(340, 206, 150),
    hsl_to_rgb(20, 206, 150),
    hsl_to_rgb(0, 206, 90),
    hsl_to_rgb(340, 206, 90),
    hsl_to_rgb(20, 206, 90),
];

const cop_colors = [
    hsl_to_rgb(50, 230, 128),
    hsl_to_rgb(40, 230, 128),
    hsl_to_rgb(60, 230, 128),
    hsl_to_rgb(50, 230, 160),
    hsl_to_rgb(40, 230, 160),
    hsl_to_rgb(60, 230, 160),
    hsl_to_rgb(50, 230, 96),
    hsl_to_rgb(40, 230, 96),
    hsl_to_rgb(60, 230, 96),
];

const heatpump_colors = [
    hsl_to_rgb(115, 128, 90),
    hsl_to_rgb(105, 128, 90),
    hsl_to_rgb(125, 128, 90),
    hsl_to_rgb(115, 128, 120),
    hsl_to_rgb(105, 128, 120),
    hsl_to_rgb(125, 128, 120),
    hsl_to_rgb(115, 128, 60),
    hsl_to_rgb(105, 128, 60),
    hsl_to_rgb(125, 128, 60),
];

const mkscene = (config: BackendConfig): SceneObject<SceneObjectState> => {
    return PanelBuilders.stat()
        .setUnit("watth")
        .setOption("graphMode", "none" as any)
        .setOption("textMode", "value_and_name" as any)
        .setOverrides((override: any) => {
            let i = 0;
            for (let id of config.heatpumps) {
                override
                    .matchFieldsWithName(`heatpump${id}.heat`)
                    .overrideColor({
                        fixedColor: heatpump_colors[i % heatpump_colors.length],
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Heatpump ${i + 1} Heat`);
                override
                    .matchFieldsWithName(`heatpump${id}.cop`)
                    .overrideColor({
                        fixedColor: cop_colors[i % cop_colors.length],
                        mode: "fixed",
                    })
                    .overrideUnit("none")
                    .overrideDisplayName(`Heatpump ${i + 1} CoP`);
                i += 1;
            }

            i = 0;
            for (let id of config.generators) {
                override
                    .matchFieldsWithName(`generator${id}.heat`)
                    .overrideColor({
                        fixedColor:
                            generator_colors[i % generator_colors.length],
                        mode: "fixed",
                    })
                    .overrideDisplayName(`Generator ${i + 1} Heat`);
                i += 1;
            }
        })
        .build();
};

const mkqueries = (config: BackendConfig): any => {
    let queries: any = [];

    for (let id of config.heatpumps) {
        queries.push({
            refId: `heatpump${id}.heat`,
            rawSql:
                `SELECT MAX(heat_wh)-MIN(heat_wh) AS \"heatpump${id}.heat\" ` +
                `FROM heatpumps ` +
                `WHERE series_id = ${id} AND $__timeFilter(time)`,
            format: "table",
        });
        queries.push({
            refId: `heatpump${id}.cop`,
            rawSql:
                `SELECT AVG(cop_pct)/100.0 AS \"heatpump${id}.cop\" ` +
                `FROM heatpumps ` +
                `WHERE series_id = ${id} AND $__timeFilter(time) ` +
                `AND cop_pct > 100`,
            format: "table",
        });
    }

    for (let id of config.generators) {
        queries.push({
            refId: `generator${id}.heat`,
            rawSql:
                // power * (1-eta_el)/eta_el * f_Hs_Hi",
                // d_runtime_s / 300 * 800 * (1-0.138)/0.138 * 1.11
                // === d_runtime_s * 2.66667 * 6.93348
                // === d_runtime_s * 18.48928
                `SELECT (MAX(runtime_s)-MIN(runtime_s)) * 18.48928 ` +
                `AS \"generator${id}.heat\" ` +
                `FROM generators ` +
                `WHERE series_id = ${id} AND $__timeFilter(time)`,
            format: "table",
        });
    }

    return queries;
};

// TODO: dedup
export const HeatStats = (config: ConfigJson): Panel => {
    const queryRunner = new SceneQueryRunner({
        datasource: {
            uid: config.datasource?.uid || "",
        },
        queries: mkqueries(config.backend || BackendConfigDefault),
    });

    return {
        query: queryRunner,
        scene: mkscene(config.backend || BackendConfigDefault),
    };
};

export let privateFunctions: any = {};
if (process.env.NODE_ENV === "test") {
    privateFunctions = {
        mkqueries,
    };
}
