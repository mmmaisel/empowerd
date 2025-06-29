import React, { Component, ReactNode } from "react";
import { Redirect, Route, Switch } from "react-router-dom";

import { ConfigJson } from "./AppConfig";
import { HomePage } from "./Home";
import pluginJson from "./plugin.json";

export enum ROUTES {
    Home = "",
    Power = "power",
    Heating = "heating",
    Weather = "weather",

    Production = "production",
    Consumption = "consumption",
    Stats = "stats",
    Details = "details",
    Solar = "solar",
    Histogram = "histogram",
    Boiler = "boiler",
    Heat = "heat",
    Temperature = "temperature",
    Humidity = "humidity",
    Rain = "rain",
    Barometer = "barometer",
    Wind = "wind",
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

    private onBack(): void {
        this.setState({ back: true });
    }

    public render(): ReactNode {
        if (this.state.back) {
            // TODO: only go one layer up
            this.setState({ back: false });
            return <Redirect to={prefixRoute(ROUTES.Home)} />;
        }

        return (
            <Switch>
                <Route
                    path={prefixRoute(ROUTES.Home)}
                    render={(props) => (
                        <HomePage
                            config={this.props}
                            backCb={this.onBack.bind(this)}
                        />
                    )}
                />
                <Redirect to={prefixRoute(ROUTES.Home)} />
            </Switch>
        );
    }
}
