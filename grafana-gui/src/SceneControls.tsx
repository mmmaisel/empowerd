import { IconName } from "@grafana/ui";
import {
    SceneControlsSpacer,
    SceneObject,
    SceneObjectState,
    SceneRefreshPicker,
    SceneTimePicker,
    SceneToolbarButton,
} from "@grafana/scenes";

export function MainControls(): Array<SceneObject<SceneObjectState>> {
    return [
        new SceneControlsSpacer(),
        new SceneTimePicker({ isOnCanvas: true }),
        new SceneRefreshPicker({ isOnCanvas: true, refresh: "5m" }),
    ];
}

export function DrilldownControls(
    onClick: () => void
): Array<SceneObject<SceneObjectState>> {
    return [
        new SceneToolbarButton({
            icon: "arrow-up" as IconName,
            onClick,
        }),
        ...MainControls(),
    ];
}
