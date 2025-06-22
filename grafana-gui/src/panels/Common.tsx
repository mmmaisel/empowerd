import { PanelMenuItem } from "@grafana/data";
import {
    EmbeddedScene,
    VizPanelBuilder,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
    VizPanelMenu,
} from "@grafana/scenes";
import { DataSourceRef } from "@grafana/schema";

import { BackendConfig, BackendConfigDefault } from "../AppConfig";

export abstract class EmpPanelBuilder {
    public config: BackendConfig;
    public ds_uid: string;
    public menu_items: PanelMenuItem[];

    constructor(
        config: BackendConfig | undefined,
        datasource: DataSourceRef | undefined = undefined,
        menu_items: PanelMenuItem[] = []
    ) {
        this.config = config || BackendConfigDefault;
        this.ds_uid = datasource?.uid || "";
        this.menu_items = menu_items;
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

    protected build_menu(builder: VizPanelBuilder<any, any>): void {
        if (this.menu_items.length !== 0) {
            builder
                .setMenu(
                    new VizPanelMenu({
                        items: this.menu_items,
                    })
                )
                .setShowMenuAlways(true);
        }
    }

    public build(): EmbeddedScene {
        return new EmbeddedScene({
            $data: this.query_runner(),
            body: this.scene(),
        });
    }
}
