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
    SceneTimePicker,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { ROUTES, prefixRoute } from "./Routes";
import { PowerPlot } from "./panels/PowerPlot";
import { PowerStats } from "./panels/PowerStats";

type HomePageProps = { config: ConfigJson };
type HomePageState = {};

export class HomePage extends Component<HomePageProps, HomePageState> {
    scene: SceneApp;

    constructor(props: HomePageProps) {
        super(props);

        this.scene = new SceneApp({
            pages: [
                new SceneAppPage({
                    title: "Empowerd Home",
                    url: prefixRoute(ROUTES.Home),
                    getScene: this.mkscene.bind(this),
                }),
            ],
        });
    }

    mkscene(): EmbeddedScene {
        let plot = PowerPlot(this.props.config);
        let stats = PowerStats(this.props.config);
        return new EmbeddedScene({
            $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
            body: new SceneCSSGridLayout({
                templateColumns: "minmax(1fr, 1fr)",
                templateRows: "5fr 1fr",
                children: [
                    new EmbeddedScene({
                        $data: plot.query,
                        body: plot.scene,
                    }),
                    new EmbeddedScene({
                        $data: stats.query,
                        body: stats.scene,
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
