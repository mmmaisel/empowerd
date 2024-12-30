import React from "react";
import { AppRootProps } from "@grafana/data";

import { Routes } from "./Routes";
import { init_i18n } from "./i18n";

export let IMG_PATH = "";
init_i18n();

export class App extends React.PureComponent<AppRootProps> {
    constructor(props: AppRootProps) {
        super(props);

        // App is singleton. Init global image path here.
        let prefix = (props.meta.defaultNavUrl || "").replace(
            /\/plugins\/empowerd\/.*$/,
            ""
        );
        IMG_PATH = `${prefix}/public/plugins/empowerd/img`;
    }

    render() {
        return <Routes {...this.props.meta.jsonData} />;
    }
}
