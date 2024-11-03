import React, { Component, ReactNode } from "react";
import { Redirect, Route, Switch } from "react-router-dom";

import { ConfigJson } from "./AppConfig";
import { HomePage } from "./Home";
import pluginJson from "./plugin.json";

export enum ROUTES {
    Home = "",
    Power = "power",
}

const PLUGIN_BASE_URL = `/a/${pluginJson.id}`;

// Prefixes the route with the base URL of the plugin
export const prefixRoute = (route: string): string => {
    if (route === "") {
        return PLUGIN_BASE_URL;
    } else {
        return `${PLUGIN_BASE_URL}/${route}`;
    }
};

type RoutesProps = ConfigJson;
type RoutesState = {
    back: boolean;
};

export class Routes extends Component<RoutesProps, RoutesState> {
    constructor(props: RoutesProps) {
        super(props);

        this.state = {
            back: false,
        };
    }

    onBack = (): void => {
        this.setState({ back: true });
    };

    render(): ReactNode {
        let cfg = JSON.parse(JSON.stringify(this.props));
        cfg.backend = {
            solars: [1, 8],
            generators: [2],
            heatpumps: [7],
        };

        if (this.state.back) {
            this.setState({ back: false });
            return <Redirect to={prefixRoute(ROUTES.Home)} />;
        }

        return (
            <Switch>
                <Route
                    path={prefixRoute(ROUTES.Home)}
                    render={(props) => (
                        <HomePage config={cfg} backCb={this.onBack} />
                    )}
                />
                <Redirect to={prefixRoute(ROUTES.Home)} />
            </Switch>
        );
    }
}
