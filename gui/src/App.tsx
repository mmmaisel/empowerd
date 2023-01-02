import React, { Component, ReactElement, ReactNode } from "react";
import "./App.scss";

import Navbar from "./Navbar";
import Config from "./Config";
import Status from "./Status";
import LoginForm from "./LoginForm";
import EmpowerdApi, { GraphQlError } from "./EmpowerdApi";

type AppState = {
    logged_in: boolean;
    active_tab: string;
};

class App extends Component<{}, AppState> {
    items: string[];
    api: EmpowerdApi;

    constructor(props: {}) {
        super(props);

        let location: string = window.location
            .toString()
            .replace(/^https?\/\/[^/]+(?::\d+)?\//, "/")
            .replace(/[^/]*$/, "");

        this.items = ["Status", "Logout"];
        this.api = new EmpowerdApi(location);
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
            (errors: GraphQlError[]) => {
                console.log(errors);
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
