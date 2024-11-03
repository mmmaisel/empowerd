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
import { PowerPlot } from "./panels/PowerPlot";
import { PowerStats } from "./panels/PowerStats";

export const PowerScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    let plot = PowerPlot(config);
    let stats = PowerStats(config);
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows: "5fr 1fr",
            children: [
                new EmbeddedScene({
                    $data: plot.query,
                    body: plot.scene,
                }),
                new EmbeddedScene({
                    $data: stats.query,
                    body: stats.scene,
                }),
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
