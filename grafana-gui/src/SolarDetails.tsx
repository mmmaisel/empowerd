import React from "react";
import {
    EmbeddedScene,
    SceneAppPage,
    SceneCSSGridLayout,
    SceneObject,
    SceneRouteMatch,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { SceneInfo } from "./Home";
import { SolarPlot } from "./panels/SolarPlot";
import { SolarStats } from "./panels/SolarStats";
import { SolarPerMonth } from "./panels/SolarPerMonth";
import { t } from "./i18n";

export class SolarDetailsScene {
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
            title: t("solar-details"),
            getScene: this.details_scene.bind(this),
        };
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

    private details_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows: "5fr 1fr 5fr",
                children: [
                    new SolarPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                    new SolarStats(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                    new SolarPerMonth(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }
}
