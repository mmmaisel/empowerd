import {
    EmbeddedScene,
    SceneAppDrilldownView,
    SceneCSSGridLayout,
    SceneRouteMatch,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { EmpScene, SceneInfo } from "./EmpScene";
import { BoilerPlot } from "./panels/BoilerPlot";
import { HeatSumStats } from "./panels/HeatSumStats";
import { HeatPlot } from "./panels/HeatPlot";
import { ROUTES } from "./Routes";
import { t } from "./i18n";

export class HeatingScene extends EmpScene {
    constructor(config: ConfigJson, backCb: () => void, route: string) {
        super(config, backCb, route);
    }

    protected drilldowns(): SceneAppDrilldownView[] {
        return [
            this.zoomDrilldown(ROUTES.Boiler),
            this.zoomDrilldown(ROUTES.Heat),
            this.zoomDrilldown(ROUTES.Stats),
        ];
    }

    protected route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        if (routeMatch.url.endsWith(ROUTES.Boiler)) {
            return {
                title: t("boiler"),
                getScene: this.boiler_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Heat)) {
            return {
                title: t("heat"),
                getScene: this.heat_scene.bind(this),
            };
        } else if (routeMatch.url.endsWith(ROUTES.Stats)) {
            return {
                title: t("stats"),
                getScene: this.stats_scene.bind(this),
            };
        } else {
            return {
                title: t("heating"),
                getScene: this.heating_scene.bind(this),
            };
        }
    }

    private boiler_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new BoilerPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private heat_scene(): EmbeddedScene {
        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr 1fr",
                children: [
                    new HeatPlot(
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
                templateRows: "1fr 1fr",
                children: [
                    new HeatSumStats(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            })
        );
    }

    private heating_scene(): EmbeddedScene {
        let templateRows = "3fr 1fr";
        let children: any = [
            new HeatPlot(
                this.config.backend,
                this.config.datasource,
                this.zoomMenu(ROUTES.Heat)
            ).build(),
            new HeatSumStats(
                this.config.backend,
                this.config.datasource,
                this.zoomMenu(ROUTES.Stats)
            ).build(),
        ];

        if (this.config.backend?.heatpumps.length !== 0) {
            templateRows = "3fr 3fr 1fr";
            children.unshift(
                new BoilerPlot(
                    this.config.backend,
                    this.config.datasource,
                    this.zoomMenu(ROUTES.Boiler)
                ).build()
            );
        }

        return this.mkscene(
            new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows,
                children,
            })
        );
    }
}
