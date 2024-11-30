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
    let hums = HumidityPlot(config);
    let temps = TemperaturePlot(config);
    let rain = RainPlot(config);
    let baro = BaroPlot(config);
    let wind = WindPlot(config);
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
                new SceneCSSGridLayout({
                    templateColumns: "1fr",
                    templateRows: "1fr 1fr 1fr",
                    children: [
                        new EmbeddedScene({
                            $data: rain.query,
                            body: rain.scene,
                        }),
                        new EmbeddedScene({
                            $data: baro.query,
                            body: baro.scene,
                        }),
                        new EmbeddedScene({
                            $data: wind.query,
                            body: wind.scene,
                        }),
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
