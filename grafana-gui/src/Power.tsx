import {
    EmbeddedScene,
    SceneCSSGridLayout,
    SceneTimeRange,
} from "@grafana/scenes";

import { ConfigJson } from "./AppConfig";
import { DrilldownControls } from "./SceneControls";
import { PowerConsumptionPlot } from "./panels/PowerConsumptionPlot";
import { PowerProductionPlot } from "./panels/PowerProductionPlot";
import { PowerStats } from "./panels/PowerStats";
import { ROUTES } from "./Routes";
import { t } from "./i18n";

export const PowerScene = (
    config: ConfigJson,
    backCb: () => void
): EmbeddedScene => {
    return new EmbeddedScene({
        $timeRange: new SceneTimeRange({ from: "now-2d", to: "now" }),
        body: new SceneCSSGridLayout({
            templateColumns: "minmax(1fr, 1fr)",
            templateRows: "5fr 5fr 2fr",
            children: [
                new PowerProductionPlot(
                    config.backend,
                    config.datasource
                ).build(),
                new PowerConsumptionPlot(
                    config.backend,
                    config.datasource
                ).build(),
                new PowerStats(config.backend, config.datasource, {
                    solar: [
                        {
                            title: t("solar-per-mon"),
                            url: `\${__url.path}/${ROUTES.Details}`,
                        },
                    ],
                }).build(),
            ],
        }),
        controls: DrilldownControls(() => {
            backCb();
        }),
    });
};
