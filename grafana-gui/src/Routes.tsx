import React, { Component, ReactNode } from "react";
import { Redirect, Route, Switch } from "react-router-dom";

import { ConfigJson } from "./AppConfig";
import { HomePage } from "./Home";
import pluginJson from "./plugin.json";

export enum ROUTES {
    Home = "home",
}

const PLUGIN_BASE_URL = `/a/${pluginJson.id}`;

// Prefixes the route with the base URL of the plugin
export const prefixRoute = (route: string): string => {
    return `${PLUGIN_BASE_URL}/${route}`;
};

type RoutesProps = ConfigJson;
type RoutesState = {};

export class Routes extends Component<RoutesProps, RoutesState> {
    render(): ReactNode {
        let cfg = JSON.parse(JSON.stringify(this.props));
        cfg.backend = {
            solars: [1, 8],
            generators: [2],
            heatpumps: [7],
        };

        return (
            <Switch>
                <Route
                    path={prefixRoute(`${ROUTES.Home}`)}
                    render={(props) => <HomePage config={cfg} />}
                />
                <Redirect to={prefixRoute(ROUTES.Home)} />
            </Switch>
        );
    }
}
