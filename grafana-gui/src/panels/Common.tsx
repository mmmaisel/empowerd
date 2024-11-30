import {
    EmbeddedScene,
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";

type PanelArgs = {
    query: SceneQueryRunner | SceneDataTransformer;
    scene: SceneObject<SceneObjectState>;
}

export class Panel {
    public query: SceneQueryRunner | SceneDataTransformer;
    public scene: SceneObject<SceneObjectState>;

    constructor(
        args: PanelArgs
    ) {
        this.query = args.query;
        this.scene = args.scene;
    }

    public to_scene(): EmbeddedScene {
        return new EmbeddedScene({
            $data: this.query,
            body: this.scene,
        });
    }
}
