import {
    EmbeddedScene,
    SceneCSSGridLayout,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { BoilerPlot } from "./panels/BoilerPlot";
import { HeatSumStats } from "./panels/HeatSumStats";
import { HeatPlot } from "./panels/HeatPlot";

export const HeatingScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    let templateRows = "3fr 1fr";
    let children: any = [
        new HeatPlot(config.backend, config.datasource).build(),
        new HeatSumStats(config.backend, config.datasource).build(),
    ];

    if (config.backend?.heatpumps.length !== 0) {
        templateRows = "3fr 3fr 1fr";
        children.unshift(
            new BoilerPlot(config.backend, config.datasource).build()
        );
    }

    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows,
            children,
        }),
        controls: DrilldownControls(() => {
            backCb();
        }),
    });
};
