import { IconName } from "@grafana/ui";
import {
    EmbeddedScene,
    SceneControlsSpacer,
    SceneCSSGridLayout,
    SceneRefreshPicker,
    SceneTimePicker,
    SceneTimeRange,
    SceneToolbarButton,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { BoilerPlot } from "./panels/BoilerPlot";
import { HeatSumStats } from "./panels/HeatSumStats";
import { HeatPlot } from "./panels/HeatPlot";

// TODO: dedup controls and embedded scene
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
                BoilerPlot(config).to_scene(),
                HeatPlot(config).to_scene(),
                // TODO: display accumulated stats
                HeatSumStats(config).to_scene(),
            ],
        }),
        controls: [
            new SceneToolbarButton({
                icon: "arrow-up" as IconName,
                onClick: () => {
                    backCb();
                },
            }),
            new SceneControlsSpacer(),
            new SceneTimePicker({ isOnCanvas: true }),
            new SceneRefreshPicker({ isOnCanvas: true, refresh: "5m" }),
        ],
    });
};
