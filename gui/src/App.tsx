import React, { Component, ReactElement, ReactNode } from "react";
import "./App.scss";

import Navbar from "./Navbar.jsx";
import Config from "./Config.jsx";
import Status from "./Status.jsx";
import LoginForm from "./LoginForm.jsx";
import WaterApi from "./WaterApi.jsx";

type AppState = {
    logged_in: boolean;
    active_tab: string;
};

class App extends Component<{}, AppState> {
    items: string[];
    api: WaterApi;

    constructor(props: {}) {
        super(props);

        let location: string = window.location
            .toString()
            .replace(/^https?\/\/[^/]+(?::\d+)?\//, "/")
            .replace(/[^/]*$/, "");

        this.items = ["Status", "Logout"];
        this.api = new WaterApi(location);
        this.state = {
            logged_in: false,
            active_tab: this.items[0],
        };
    }

    onLogin = (): void => {
        this.setState({ logged_in: true });
    };

    onLogout = (): void => {
        this.api.logout(
            () => {
                this.setState({ logged_in: false });
            },
            (error: string) => {
                console.log(error);
                alert("Logout failed");
            }
        );
    };

    onTab = (which: string): void => {
        if (which === "Logout") {
            this.onLogout();
        } else {
            this.setState({ active_tab: which });
        }
    };

    render(): ReactNode {
        let content: ReactElement;
        if (this.state.logged_in) {
            if (this.state.active_tab === "Status") {
                content = <Status api={this.api} />;
            } else if (this.state.active_tab === "Config") {
                content = <Config api={this.api} />;
            } else {
                throw "Invalid active_tab";
            }
            return (
                <div>
                    <Navbar
                        items={this.items}
                        onTab={this.onTab}
                        active_tab={this.state.active_tab}
                    />
                    {content}
                </div>
            );
        } else {
            return (
                <div className="loginScreen">
                    <LoginForm api={this.api} onLogin={this.onLogin} />
                </div>
            );
        }
    }
}

export default App;
