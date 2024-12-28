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
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows: "3fr 3fr 1fr",
            children: [
                new BoilerPlot(config.backend, config.datasource).build(),
                new HeatPlot(config.backend, config.datasource).build(),
                new HeatSumStats(config.backend, config.datasource).build(),
            ],
        }),
        controls: DrilldownControls(() => {
            backCb();
        }),
    });
};
