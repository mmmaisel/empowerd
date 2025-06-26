import {
    EmbeddedScene,
    SceneAppDrilldownView,
    SceneCSSGridLayout,
    SceneRouteMatch,
} from "@grafana/scenes";

import { BaroPlot } from "./panels/BaroPlot";
import { ConfigJson } from "./AppConfig";
import { EmpScene, SceneInfo } from "./EmpScene";
import { HumidityPlot } from "./panels/HumidityPlot";
import { RainPlot } from "./panels/RainPlot";
import { ROUTES } from "./Routes";
import { t } from "./i18n";
import { TemperaturePlot } from "./panels/TemperaturePlot";
import { WindPlot } from "./panels/WindPlot";
import { WeatherStats } from "./panels/WeatherStats";

export class WeatherScene extends EmpScene {
    constructor(config: ConfigJson, backCb: () => void, route: string) {
        super(config, backCb, route);
    }

    protected drilldowns(): SceneAppDrilldownView[] {
        return [
            this.zoomDrilldown(ROUTES.Temperature),
            this.zoomDrilldown(ROUTES.Humidity),
            this.zoomDrilldown(ROUTES.Rain),
            this.zoomDrilldown(ROUTES.Barometer),
            this.zoomDrilldown(ROUTES.Wind),
        ];
    }

    protected route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        if (routeMatch.url.endsWith(ROUTES.Temperature)) {
            return {
                title: t("temperature"),
                getScene: this.temperature_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Humidity)) {
            return {
                title: t("humidity"),
                getScene: this.humidity_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Rain)) {
            return {
                title: t("rain"),
                getScene: this.rain_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Barometer)) {
            return {
                title: t("barometer"),
                getScene: this.barometer_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Wind)) {
            return {
                title: t("wind"),
                getScene: this.wind_scene.bind(this),
            };
        } else {
            return {
                title: t("weather"),
                getScene: this.weather_scene.bind(this),
            };
        }
    }

    private temperature_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new TemperaturePlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private humidity_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new HumidityPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private rain_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "3fr 1fr",
                children: [
                    new RainPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                    new WeatherStats(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private barometer_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new BaroPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private wind_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new WindPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private weather_scene(): EmbeddedScene {
        let children = [
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr 1fr",
                children: [
                    new TemperaturePlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Temperature)
                    ).build(),
                    new HumidityPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Humidity)
                    ).build(),
                ],
            }),
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "3fr 3fr 3fr 1fr",
                children: [
                    new RainPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Rain)
                    ).build(),
                    new BaroPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Barometer)
                    ).build(),
                    new WindPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Wind)
                    ).build(),
                    new WeatherStats(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            }),
        ];

        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr 1fr",
                templateRows: "1fr",
                children,
            })
        );
    }
}
