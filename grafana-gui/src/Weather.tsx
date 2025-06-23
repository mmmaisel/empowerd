import React from "react";
import { PanelMenuItem } from "@grafana/data";
import {
    EmbeddedScene,
    SceneAppPage,
    SceneAppDrilldownView,
    SceneCSSGridLayout,
    SceneObject,
    SceneRouteMatch,
    SceneTimeRange,
} from "@grafana/scenes";

import { BaroPlot } from "./panels/BaroPlot";
import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { HumidityPlot } from "./panels/HumidityPlot";
import { RainPlot } from "./panels/RainPlot";
import { ROUTES, prefixRoute } from "./Routes";
import { SceneInfo } from "./Home";
import { t } from "./i18n";
import { TemperaturePlot } from "./panels/TemperaturePlot";
import { WindPlot } from "./panels/WindPlot";
import { WeatherStats } from "./panels/WeatherStats";

export class WeatherScene {
    config: ConfigJson;
    backCb: () => void;

    constructor(config: ConfigJson, backCb: () => void) {
        this.config = config;
        this.backCb = backCb;
    }

    private drilldown(route: ROUTES): SceneAppDrilldownView {
        return {
            routePath: prefixRoute(`${ROUTES.Weather}/${route}`),
            getPage: this.getPage.bind(this),
        };
    }

    public getPage(routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let { title, getScene } = this.route(routeMatch);

        return new SceneAppPage({
            url: routeMatch.url,
            title,
            renderTitle: () => {
                return <></>;
            },
            getParentPage: () => parent,
            getScene,
            drilldowns: [
                this.drilldown(ROUTES.Temperature),
                this.drilldown(ROUTES.Humidity),
                this.drilldown(ROUTES.Rain),
                this.drilldown(ROUTES.Barometer),
                this.drilldown(ROUTES.Wind),
            ],
        });
    }

    private route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
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

    private zoom_menu(route: string): PanelMenuItem[] {
        return [
            {
                text: t("zoom"),
                href: prefixRoute(route),
                iconClassName: "eye",
            },
        ];
    }

    private mkscene(body: SceneObject): EmbeddedScene {
        return new EmbeddedScene({
            $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
            body,
            controls: DrilldownControls(() => {
                this.backCb();
            }),
        });
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
                        this.zoom_menu(
                            `${ROUTES.Weather}/${ROUTES.Temperature}`
                        )
                    ).build(),
                    new HumidityPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoom_menu(`${ROUTES.Weather}/${ROUTES.Humidity}`)
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
                        this.zoom_menu(`${ROUTES.Weather}/${ROUTES.Rain}`)
                    ).build(),
                    new BaroPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoom_menu(`${ROUTES.Weather}/${ROUTES.Barometer}`)
                    ).build(),
                    new WindPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoom_menu(`${ROUTES.Weather}/${ROUTES.Wind}`)
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
