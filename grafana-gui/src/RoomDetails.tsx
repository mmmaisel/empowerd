import {
    EmbeddedScene,
    SceneAppDrilldownView,
    SceneCSSGridLayout,
    SceneRouteMatch,
    SceneVariableSet,
    CustomVariable,
    VariableValueSelectors,
} from "@grafana/scenes";

import { DrilldownControls } from "./SceneControls";
import { ConfigJson } from "./AppConfig";
import { EmpScene, SceneInfo } from "./EmpScene";
import { RoomPlot } from "./panels/RoomPlot";
import { t } from "./i18n";

export class RoomDetailsScene extends EmpScene {
    constructor(config: ConfigJson, backCb: () => void, route: string) {
        super(config, backCb, route);
    }

    protected drilldowns(): SceneAppDrilldownView[] {
        return [];
    }

    protected route(routeMatch: SceneRouteMatch<{}>): SceneInfo {
        return {
            title: t("rooms"),
            getScene: this.room_scene.bind(this),
        };
    }

    private room_scene(): EmbeddedScene {
        let labels = this.config.backend!.labels;
        // Custom variable is configured though a fucking string instead
        // of a nicely typed key-value array :(
        let query = [
            `${t("outside")} : out`,
            `${t("inside")} : in`,
            ...Object.keys(labels)
                .filter((k: string) => !!(labels as any)[k])
                .map((k: string) => `${(labels as any)[k]} : ${k}`),
        ].join(", ");
        let rooms = new CustomVariable({
            name: "room",
            label: "Raum",
            query,
            allowCustomValue: false,
            isMulti: false,
        });

        let controls = DrilldownControls(() => {
            this.backCb();
        });
        controls.splice(1, 0, new VariableValueSelectors({}));

        return new EmbeddedScene({
            $timeRange: this.timeRange(),
            $variables: new SceneVariableSet({
                variables: [rooms],
            }),
            body: new SceneCSSGridLayout({
                templateColumns: "1fr",
                templateRows: "1fr",
                children: [
                    new RoomPlot(
                        this.config.backend,
                        this.config.datasource
                    ).build(),
                ],
            }),
            controls,
        });
    }
}
