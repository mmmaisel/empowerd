import {
    EmbeddedScene,
    SceneAppDrilldownView,
    SceneCSSGridLayout,
    SceneRouteMatch,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { EmpScene, SceneInfo } from "./EmpScene";
import { PowerConsumptionPlot } from "./panels/PowerConsumptionPlot";
import { PowerProductionPlot } from "./panels/PowerProductionPlot";
import { PowerStats } from "./panels/PowerStats";
import { ROUTES, prefixRoute } from "./Routes";
import { SolarDetailsScene } from "./SolarDetails";
import { t } from "./i18n";

export class PowerScene extends EmpScene {
    private details: SolarDetailsScene;

    constructor(config: ConfigJson, backCb: () => void, route: string) {
        super(config, backCb, route);
        this.details = new SolarDetailsScene(
            config,
            backCb,
            `${this.baseRoute}/${ROUTES.Details}`
        );
    }

    protected drilldowns(): SceneAppDrilldownView[] {
        return [
            this.zoomDrilldown(ROUTES.Production),
            this.zoomDrilldown(ROUTES.Consumption),
            this.zoomDrilldown(ROUTES.Stats),
            {
                routePath: prefixRoute(`${this.baseRoute}/${ROUTES.Details}`),
                getPage: this.details.getPage.bind(this.details),
            },
        ];
    }

    protected route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        if (routeMatch.url.endsWith(ROUTES.Production)) {
            return {
                title: t("pwr-prod"),
                getScene: this.production_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Consumption)) {
            return {
                title: t("pwr-cons"),
                getScene: this.consumption_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Stats)) {
            return {
                title: t("stats"),
                getScene: this.stats_scene.bind(this),
            };
        } else {
            return {
                title: t("pwr-prod-and-cons"),
                getScene: this.power_scene.bind(this),
            };
        }
    }

    private production_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new PowerProductionPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private consumption_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new PowerConsumptionPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private stats_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new PowerStats(
                        this.config.backend,
                        this.config.datasource,
                        [],
                        {
                            solar: [],
                        }
                    ).build(),
                ],
            })
        );
    }

    private power_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows: "5fr 5fr 2fr",
                children: [
                    new PowerProductionPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Production)
                    ).build(),
                    new PowerConsumptionPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Consumption)
                    ).build(),
                    new PowerStats(
                        this.config.backend,
                        this.config.datasource,
                        [
                            ...this.zoomMenu(ROUTES.Stats),
                            {
                                text: t("details"),
                                href: prefixRoute(
                                    `${this.baseRoute}/${ROUTES.Details}`
                                ),
                                iconClassName: "bolt",
                            },
                        ],
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
