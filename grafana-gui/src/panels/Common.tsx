import {
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";

export type Panel = {
    query: SceneQueryRunner;
    scene: SceneObject<SceneObjectState>;
};
