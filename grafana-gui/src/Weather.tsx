import React from "react";
import {
    EmbeddedScene,
    SceneAppPage,
    SceneCSSGridLayout,
    SceneRouteMatch,
    SceneTimeRange,
} from "@grafana/scenes";

import { BaroPlot } from "./panels/BaroPlot";
import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { HumidityPlot } from "./panels/HumidityPlot";
import { RainPlot } from "./panels/RainPlot";
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
        });
    }

    private route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        return {
            title: t("weather"),
            getScene: this.weather_scene.bind(this),
        };
    }

    private weather_scene(): EmbeddedScene {
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
                                this.config.backend,
                                this.config.datasource
                            ).build(),
                            new HumidityPlot(
                                this.config.backend,
                                this.config.datasource
                            ).build(),
                        ],
                    }),
                    new SceneCSSGridLayout({
                        templateColumns: "1fr",
                        templateRows: "3fr 3fr 3fr 1fr",
                        children: [
                            new RainPlot(
                                this.config.backend,
                                this.config.datasource
                            ).build(),
                            new BaroPlot(
                                this.config.backend,
                                this.config.datasource
                            ).build(),
                            new WindPlot(
                                this.config.backend,
                                this.config.datasource
                            ).build(),
                            new WeatherStats(
                                this.config.backend,
                                this.config.datasource
                            ).build(),
                        ],
                    }),
                ],
            }),
            controls: DrilldownControls(() => {
                this.backCb();
            }),
        });
    }
}
