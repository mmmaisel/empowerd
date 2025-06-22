import React from "react";
import {
    EmbeddedScene,
    SceneAppPage,
    SceneCSSGridLayout,
    SceneRouteMatch,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { BoilerPlot } from "./panels/BoilerPlot";
import { HeatSumStats } from "./panels/HeatSumStats";
import { HeatPlot } from "./panels/HeatPlot";
import { SceneInfo } from "./Home";
import { t } from "./i18n";

export class HeatingScene {
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
            title: t("heating"),
            getScene: this.heating_scene.bind(this),
        };
    }

    private heating_scene(): EmbeddedScene {
        let templateRows = "3fr 1fr";
        let children: any = [
            new HeatPlot(this.config.backend, this.config.datasource).build(),
            new HeatSumStats(
                this.config.backend,
                this.config.datasource
            ).build(),
        ];

        if (this.config.backend?.heatpumps.length !== 0) {
            templateRows = "3fr 3fr 1fr";
            children.unshift(
                new BoilerPlot(
                    this.config.backend,
                    this.config.datasource
                ).build()
            );
        }

        return new EmbeddedScene({
            $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
            body: new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows,
                children,
            }),
            controls: DrilldownControls(() => {
                this.backCb();
            }),
        });
    }
}
