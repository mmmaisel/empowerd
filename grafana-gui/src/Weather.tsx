import {
    EmbeddedScene,
    SceneCSSGridLayout,
    SceneTimeRange,
} from "@grafana/scenes";

import { BaroPlot } from "./panels/BaroPlot";
import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { HumidityPlot } from "./panels/HumidityPlot";
import { RainPlot } from "./panels/RainPlot";
import { TemperaturePlot } from "./panels/TemperaturePlot";
import { WindPlot } from "./panels/WindPlot";
import { WeatherStats } from "./panels/WeatherStats";

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
                        new TemperaturePlot(
                            config.backend,
                            config.datasource
                        ).build(),
                        new HumidityPlot(
                            config.backend,
                            config.datasource
                        ).build(),
                    ],
                }),
                new SceneCSSGridLayout({
                    templateColumns: "1fr",
                    templateRows: "3fr 3fr 3fr 1fr",
                    children: [
                        new RainPlot(config.backend, config.datasource).build(),
                        new BaroPlot(config.backend, config.datasource).build(),
                        new WindPlot(config.backend, config.datasource).build(),
                        new WeatherStats(
                            config.backend,
                            config.datasource
                        ).build(),
                    ],
                }),
            ],
        }),
        controls: DrilldownControls(() => {
            backCb();
        }),
    });
};
