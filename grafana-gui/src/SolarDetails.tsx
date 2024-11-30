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
import { SolarPerMonth } from "./panels/SolarPerMonth";

export const SolarDetailsScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-3y", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows: "5fr 1fr",
            children: [
                SolarPerMonth(config).to_scene(),
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
