import React, { Component, ReactNode } from "react";
import { config } from "@grafana/runtime";
import { Alert } from "@grafana/ui";
import {
    SceneApp,
    SceneAppPage,
    EmbeddedScene,
    SceneCSSGridLayout,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { Control } from "./control/Control";
import { ROUTES, prefixRoute } from "./Routes";
import { Overview } from "./panels/Overview";
import { PowerScene } from "./Power";
import { HeatingScene } from "./Heating";
import { t } from "./i18n";
import { MainControls } from "./SceneControls";
import { WeatherScene } from "./Weather";

type HomePageProps = {
    config: ConfigJson;
    backCb: () => void;
};
type HomePageState = {};

export class HomePage extends Component<HomePageProps, HomePageState> {
    power: PowerScene;
    heating: HeatingScene;
    weather: WeatherScene;
    scene: SceneApp;

    constructor(props: HomePageProps) {
        super(props);

        this.power = new PowerScene(props.config, props.backCb);
        this.heating = new HeatingScene(
            props.config,
            props.backCb,
            ROUTES.Heating
        );
        this.weather = new WeatherScene(
            props.config,
            props.backCb,
            ROUTES.Weather
        );

        this.scene = new SceneApp({
            pages: [
                new SceneAppPage({
                    title: t("overview"),
                    renderTitle: () => {
                        return <></>;
                    },
                    url: prefixRoute(ROUTES.Home),
                    getScene: this.mkscene.bind(this),
                    drilldowns: [
                        {
                            routePath: prefixRoute(ROUTES.Power),
                            getPage: this.power.getPage.bind(this.power),
                        },
                        {
                            routePath: prefixRoute(ROUTES.Heating),
                            getPage: this.heating.getPage.bind(this.heating),
                        },
                        {
                            routePath: prefixRoute(ROUTES.Weather),
                            getPage: this.weather.getPage.bind(this.weather),
                        },
                    ],
                }),
            ],
        });
    }

    private mkscene(): EmbeddedScene {
        let templateRows = "1fr";
        let children: any = [
            new Overview(
                this.props.config.backend,
                this.props.config.datasource,
                {
                    power: [
                        {
                            title: t("pwr-prod-and-cons"),
                            url: `\${__url.path}/${ROUTES.Power}`,
                        },
                    ],
                    heatpump: [
                        {
                            title: t("heating"),
                            url: `\${__url.path}/${ROUTES.Heating}`,
                        },
                    ],
                    weather: [
                        {
                            title: t("weather"),
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

    public render(): ReactNode {
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
