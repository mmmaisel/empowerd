import React, { Component } from "react";
import "./App.scss";

import Navbar from "./Navbar.jsx";
import Config from "./Config.jsx";
import Status from "./Status.jsx";
import LoginForm from "./LoginForm.jsx";
import WaterApi from "./WaterApi.jsx";

class App extends Component {
    constructor(props) {
        super(props);
        this.items = ["Status", "Config", "Logout"];
        this.api = new WaterApi();
        this.state = {
            logged_in: false,
            active_tab: this.items[0],
        };
    }

    onLogin = () => {
        this.setState({ logged_in: true });
    };

    onLogout = () => {
        this.api.logout(
            (response) => {
                this.setState({ logged_in: false });
            },
            (error) => {
                console.log(error);
                alert("Logout failed");
            }
        );
    };

    onTab = (which) => {
        if (which === "Logout") {
            this.onLogout();
        } else {
            this.setState({ active_tab: which });
        }
    };

    render() {
        let content;
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
