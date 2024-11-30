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

import { BaroPlot } from "./panels/BaroPlot";
import { ConfigJson } from "./AppConfig";
import { HumidityPlot } from "./panels/HumidityPlot";
import { RainPlot } from "./panels/RainPlot";
import { TemperaturePlot } from "./panels/TemperaturePlot";
import { WindPlot } from "./panels/WindPlot";

// TODO: dedup controls and embedded scene
export const WeatherScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "1fr 1fr",
            templateRows: "1fr",
            children: [
                new SceneCSSGridLayout({
                    templateColumns: "1fr",
                    templateRows: "1fr 1fr",
                    children: [
                        TemperaturePlot(config).to_scene(),
                        HumidityPlot(config).to_scene(),
                    ],
                }),
                new SceneCSSGridLayout({
                    templateColumns: "1fr",
                    templateRows: "1fr 1fr 1fr",
                    children: [
                        RainPlot(config).to_scene(),
                        BaroPlot(config).to_scene(),
                        WindPlot(config).to_scene(),
                    ],
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
