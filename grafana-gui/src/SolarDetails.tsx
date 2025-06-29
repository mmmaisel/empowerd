import {
    EmbeddedScene,
    SceneAppDrilldownView,
    SceneCSSGridLayout,
    SceneRouteMatch,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { EmpScene, SceneInfo } from "./EmpScene";
import { ROUTES } from "./Routes";
import { SolarPlot } from "./panels/SolarPlot";
import { SolarStats } from "./panels/SolarStats";
import { SolarPerMonth } from "./panels/SolarPerMonth";
import { t } from "./i18n";

export class SolarDetailsScene extends EmpScene {
    constructor(config: ConfigJson, backCb: () => void, route: string) {
        super(config, backCb, route);
    }

    protected drilldowns(): SceneAppDrilldownView[] {
        return [
            this.zoomDrilldown(ROUTES.Solar),
            this.zoomDrilldown(ROUTES.Stats),
            this.zoomDrilldown(ROUTES.Histogram),
        ];
    }

    protected route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        if (routeMatch.url.endsWith(ROUTES.Solar)) {
            return {
                title: t("solar"),
                getScene: this.solar_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Stats)) {
            return {
                title: t("stats"),
                getScene: this.stats_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Histogram)) {
            return {
                title: t("solar-per-mon"),
                getScene: this.histogram_scene.bind(this),
            };
        } else {
            return {
                title: t("solar-details"),
                getScene: this.details_scene.bind(this),
            };
        }
    }

    private solar_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new SolarPlot(
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
                    new SolarStats(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private histogram_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new SolarPerMonth(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private details_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows: "5fr 1fr 5fr",
                children: [
                    new SolarPlot(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Solar)
                    ).build(),
                    new SolarStats(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Stats)
                    ).build(),
                    new SolarPerMonth(
                        this.config.backend,
                        this.config.datasource,
                        this.zoomMenu(ROUTES.Histogram)
                    ).build(),
                ],
            })
        );
    }
}
