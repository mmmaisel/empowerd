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
import { PowerConsumptionPlot } from "./panels/PowerConsumptionPlot";
import { PowerProductionPlot } from "./panels/PowerProductionPlot";
import { PowerStats } from "./panels/PowerStats";
import { ROUTES, prefixRoute } from "./Routes";
import { SceneInfo } from "./EmpScene";
import { SolarDetailsScene } from "./SolarDetails";
import { t } from "./i18n";

export class PowerScene {
    config: ConfigJson;
    backCb: () => void;
    details: SolarDetailsScene;

    constructor(config: ConfigJson, backCb: () => void) {
        this.config = config;
        this.backCb = backCb;
        this.details = new SolarDetailsScene(config, backCb);
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
                {
                    routePath: prefixRoute(`${ROUTES.Power}/${ROUTES.Details}`),
                    getPage: this.details.getPage.bind(this.details),
                },
            ],
        });
    }

    private route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        return {
            title: t("pwr-prod-and-cons"),
            getScene: this.power_scene.bind(this),
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

    private power_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows: "5fr 5fr 2fr",
                children: [
                    new PowerProductionPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                    new PowerConsumptionPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                    new PowerStats(
                        this.config.backend,
                        this.config.datasource,
                        [],
                        {
                            solar: [
                                {
                                    title: t("solar-per-mon"),
                                    url: `\${__url.path}/${ROUTES.Details}`,
                                },
                            ],
                        }
                    ).build(),
                ],
            })
        );
    }
}
