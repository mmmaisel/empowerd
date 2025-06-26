import React from "react";
import { PanelMenuItem } from "@grafana/data";
import {
    EmbeddedScene,
    SceneAppPage,
    SceneAppDrilldownView,
    SceneObject,
    SceneRouteMatch,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { ROUTES, prefixRoute } from "./Routes";
import { t } from "./i18n";

export type SceneInfo = {
    title: string;
    getScene: () => EmbeddedScene;
};

export abstract class EmpScene {
    protected config: ConfigJson;
    protected backCb: () => void;
    protected baseRoute: string;

    constructor(config: ConfigJson, backCb: () => void, baseRoute: string) {
        this.config = config;
        this.backCb = backCb;
        this.baseRoute = baseRoute;
    }

    protected abstract drilldowns(): SceneAppDrilldownView[];
    protected abstract route(routeMatch: SceneRouteMatch<{}>): SceneInfo;

    protected timeRange(): SceneTimeRange {
        return new SceneTimeRange({ from: "now-2d", to: "now" });
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
            drilldowns: this.drilldowns(),
        });
    }

    protected zoomDrilldown(route: ROUTES): SceneAppDrilldownView {
        return {
            routePath: prefixRoute(`${this.baseRoute}/${route}`),
            getPage: this.getPage.bind(this),
        };
    }

    protected zoomMenu(route: ROUTES): PanelMenuItem[] {
        return [
            {
                text: t("zoom"),
                href: prefixRoute(`${this.baseRoute}/${route}`),
                iconClassName: "eye",
            },
        ];
    }

    protected mkscene(body: SceneObject): EmbeddedScene {
        return new EmbeddedScene({
            $timeRange: this.timeRange(),
            body,
            controls: DrilldownControls(() => {
                this.backCb();
            }),
        });
    }
}
