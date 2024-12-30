import React, { Component, ReactNode } from "react";
import { config } from "@grafana/runtime";
import { Alert } from "@grafana/ui";
import {
    SceneApp,
    SceneAppPage,
    EmbeddedScene,
    SceneCSSGridLayout,
    SceneRouteMatch,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { Control } from "./control/Control";
import { ROUTES, prefixRoute } from "./Routes";
import { Overview } from "./panels/Overview";
import { PowerScene } from "./Power";
import { HeatingScene } from "./Heating";
import { SolarDetailsScene } from "./SolarDetails";
import { MainControls } from "./SceneControls";
import { WeatherScene } from "./Weather";

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
                    title: "Overview",
                    renderTitle: () => {
                        return <></>;
                    },
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
                        {
                            routePath: prefixRoute(ROUTES.Weather),
                            getPage: this.mkweather.bind(this),
                        },
                    ],
                }),
            ],
        });
    }

    mkscene(): EmbeddedScene {
        let templateRows = "1fr";
        let children: any = [
            new Overview(
                this.props.config.backend,
                this.props.config.datasource,
                {
                    power: [
                        {
                            title: "Power Production and Consumption",
                            url: `\${__url.path}/${ROUTES.Power}`,
                        },
                    ],
                    heatpump: [
                        {
                            title: "Heating",
                            url: `\${__url.path}/${ROUTES.Heating}`,
                        },
                    ],
                    weather: [
                        {
                            title: "Weather",
                            url: `\${__url.path}/${ROUTES.Weather}`,
                        },
                    ],
                }
            ).build(),
        ];

        if (this.props.config.backend?.controls) {
            templateRows = "1fr 2fr";
            children.push(
                new Control({ apiLocation: this.props.config.apiLocation })
            );
        }

        return new EmbeddedScene({
            $timeRange: new SceneTimeRange({ from: "now-1h", to: "now" }),
            body: new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows,
                children,
            }),
            controls: MainControls(),
        });
    }

    mkpower(_routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let props = this.props;
        return new SceneAppPage({
            url: prefixRoute(ROUTES.Power),
            title: `Power Production and Consumption`,
            renderTitle: () => {
                return <></>;
            },
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
            renderTitle: () => {
                return <></>;
            },
            getParentPage: () => parent,
            getScene: (_routeMatch: SceneRouteMatch<{}>) =>
                HeatingScene(props.config, props.backCb),
        });
    }

    mkweather(_routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let props = this.props;
        return new SceneAppPage({
            url: prefixRoute(ROUTES.Weather),
            title: `Weather`,
            renderTitle: () => {
                return <></>;
            },
            getParentPage: () => parent,
            getScene: (_routeMatch: SceneRouteMatch<{}>) =>
                WeatherScene(props.config, props.backCb),
        });
    }

    mkdetails(_routeMatch: SceneRouteMatch<{}>, parent: any): SceneAppPage {
        let props = this.props;
        return new SceneAppPage({
            url: prefixRoute(`${ROUTES.Power}/${ROUTES.Details}`),
            title: `Solar Details`,
            renderTitle: () => {
                return <></>;
            },
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
