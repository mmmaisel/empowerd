import {
    EmbeddedScene,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";
import { DataSourceRef } from "@grafana/schema";

import { BackendConfig, BackendConfigDefault } from "../AppConfig";

export abstract class EmpPanelBuilder {
    public config: BackendConfig;
    public ds_uid: string;

    constructor(
        config: BackendConfig | undefined,
        datasource: DataSourceRef | undefined = undefined
    ) {
        this.config = config || BackendConfigDefault;
        this.ds_uid = datasource?.uid || "";
    }

    public abstract scene(): SceneObject<SceneObjectState>;
    public abstract queries(): any[];

    protected query_runner(): SceneQueryRunner {
        return new SceneQueryRunner({
            datasource: {
                uid: this.ds_uid,
            },
            queries: this.queries(),
        });
    }

    public build(): EmbeddedScene {
        return new EmbeddedScene({
            $data: this.query_runner(),
            body: this.scene(),
        });
    }
}
