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
import { SolarPlot } from "./panels/SolarPlot";
import { SolarStats } from "./panels/SolarStats";
import { SolarPerMonth } from "./panels/SolarPerMonth";

export const SolarDetailsScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows: "5fr 1fr 5fr",
            children: [
                new SolarPlot(config.backend, config.datasource).build(),
                new SolarStats(config.backend, config.datasource).build(),
                new SolarPerMonth(config.backend, config.datasource).build(),
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
