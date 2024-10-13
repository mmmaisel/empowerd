import {
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";

export type Panel = {
    query: SceneQueryRunner | SceneDataTransformer;
    scene: SceneObject<SceneObjectState>;
};
