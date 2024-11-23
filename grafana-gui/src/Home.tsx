import React, { Component, ReactNode } from "react";
import { config } from "@grafana/runtime";
import { Alert } from "@grafana/ui";
import {
    SceneApp,
    SceneAppPage,
    EmbeddedScene,
    SceneControlsSpacer,
    SceneCSSGridLayout,
    SceneRefreshPicker,
    SceneRouteMatch,
    SceneTimePicker,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { ROUTES, prefixRoute } from "./Routes";
import { Overview } from "./panels/Overview";
import { PowerScene } from "./Power";
import { HeatingScene } from "./Heating";
import { SolarDetailsScene } from "./SolarDetails";

type HomePageProps = {
    config: ConfigJson;
    backCb: () => void;
};
type HomePageState = {};

export class HomePage extends Component<HomePageProps, HomePageState> {
    scene: SceneApp;

    constructor(props: HomePageProps) {
        super(props);

        this.scene = new SceneApp({
            pages: [
                new SceneAppPage({
                    title: "Empowerd",
                    url: prefixRoute(ROUTES.Home),
                    getScene: this.mkscene.bind(this),
                    drilldowns: [
                        {
                            routePath: prefixRoute(ROUTES.Power),
                            getPage: this.mkpower.bind(this),
                        },
                        {
                            routePath: prefixRoute(ROUTES.Heating),
                            getPage: this.mkheating.bind(this),
                        },
                    ],
                }),
            ],
        });
    }

    mkscene(): EmbeddedScene {
        let overview = Overview(this.props.config, {
            power: [
                {
                    title: "Power",
                    url: `\${__url.path}/${ROUTES.Power}`,
                },
            ],
            heatpump: [
                {
                    title: "Heating",
                    url: `\${__url.path}/${ROUTES.Heating}`,
                },
            ],
        });
        return new EmbeddedScene({
            $timeRange: new SceneTimeRange({ from: "now-1h", to: "now" }),
            body: new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows: "5fr 1fr",
                children: [
                    new EmbeddedScene({
                        $data: overview.query,
                        body: overview.scene,
                    }),
                ],
            }),
            controls: [
                new SceneControlsSpacer(),
                new SceneTimePicker({ isOnCanvas: true }),
                new SceneRefreshPicker({ isOnCanvas: true, refresh: "5m" }),
            ],
        });
    }

    mkpower(_routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let props = this.props;
        return new SceneAppPage({
            url: prefixRoute(ROUTES.Power),
            title: `Power`,
            getParentPage: () => parent,
            getScene: (_routeMatch: SceneRouteMatch<{}>) =>
                PowerScene(props.config, props.backCb),
            drilldowns: [
                {
                    routePath: prefixRoute(`${ROUTES.Power}/${ROUTES.Details}`),
                    getPage: this.mkdetails.bind(this),
                },
            ],
        });
    }

    mkheating(_routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let props = this.props;
        return new SceneAppPage({
            url: prefixRoute(ROUTES.Heating),
            title: `Heating`,
            getParentPage: () => parent,
            getScene: (_routeMatch: SceneRouteMatch<{}>) =>
                HeatingScene(props.config, props.backCb),
        });
    }

    mkdetails(_routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let props = this.props;
        return new SceneAppPage({
            url: prefixRoute(`${ROUTES.Power}/${ROUTES.Details}`),
            title: `Solar Details`,
            getParentPage: () => parent,
            getScene: (_routeMatch: SceneRouteMatch<{}>) =>
                SolarDetailsScene(props.config, props.backCb),
        });
    }

    render(): ReactNode {
        return (
            <>
                {!config.featureToggles.topnav && (
                    <Alert title="Missing topnav feature toggle">
                        Scenes are designed to work with the new navigation
                        wrapper that will be standard in Grafana 10
                    </Alert>
                )}

                <this.scene.Component model={this.scene} />
            </>
        );
    }
}
