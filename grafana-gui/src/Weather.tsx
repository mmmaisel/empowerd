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
import { HumidityPlot } from "./panels/HumidityPlot";
import { TemperaturePlot } from "./panels/TemperaturePlot";

// TODO: dedup controls and embedded scene
export const WeatherScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    let hums = HumidityPlot(config);
    let temps = TemperaturePlot(config);
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows: "3fr 3fr 1fr",
            children: [
                new EmbeddedScene({
                    $data: temps.query,
                    body: temps.scene,
                }),
                new EmbeddedScene({
                    $data: hums.query,
                    body: hums.scene,
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
