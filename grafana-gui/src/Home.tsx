import React, { Component, ReactNode } from "react";
import { config } from "@grafana/runtime";
import { Alert } from "@grafana/ui";
import {
    SceneApp,
    SceneAppPage,
    EmbeddedScene,
    PanelBuilders,
    SceneFlexItem,
    SceneFlexLayout,
    SceneQueryRunner,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { ROUTES, prefixRoute } from "./Routes";

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
        const queryRunner = new SceneQueryRunner({
            datasource: {
                uid: this.props.config.datasource?.uid || "",
            },
            queries: [
                {
                    refId: "A",
                    rawSql: "SELECT MAX(energy_wh) FROM simple_meters WHERE series_id = 1",
                    format: "table",
                },
            ],
        });

        return new EmbeddedScene({
            $data: queryRunner,
            $timeRange: new SceneTimeRange({ from: "now-5m", to: "now" }),
            body: new SceneFlexLayout({
                children: [
                    new SceneFlexItem({
                        minHeight: 300,
                        body: PanelBuilders.stat()
                            .setTitle("PSQL test")
                            .setUnit("watth")
                            .build(),
                    }),
                ],
            }),
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
