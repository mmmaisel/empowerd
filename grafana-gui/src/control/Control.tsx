import React, { Component, ReactNode } from "react";
import { SceneObjectBase, SceneObjectState } from "@grafana/scenes";
import { EmpowerdApi, GraphQlError } from "./EmpowerdApi";
import { LoginForm } from "./LoginForm";
import { SwitchesPanel } from "./SwitchesPanel";
import "./Control.scss";

// TODO: load control charge switch
// TODO: available power sliders

type ControlImplProps = {};
type ControlImplState = {
    logged_in: boolean;
};

class ControlImpl extends Component<ControlImplProps, ControlImplState> {
    private api: EmpowerdApi;

    constructor(props: {}) {
        super(props);

        // TODO: API url
        /*let location: string = window.location
            .toString()
            .replace(/^https?\/\/[^/]+(?::\d+)?\//, "/")
            .replace(/[^/]*$/, "");*/

        this.api = new EmpowerdApi("/empowerd/graphql");
        this.state = {
            logged_in: false,
        };
    }

    public onLogin(): void {
        this.setState({ logged_in: true });
    }

    public onLogout(): void {
        this.api.logout(
            () => {
                this.setState({ logged_in: false });
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
                alert("Logout failed");
            }
        );
    }

    public render(): ReactNode {
        let content = null;

        if (this.state.logged_in) {
            content = (
                <>
                    <SwitchesPanel api={this.api} />
                    <button
                        className="dialogButton"
                        style={{ height: "2.5em" }}
                        onClick={this.onLogout.bind(this)}
                    >
                        Logout
                    </button>
                </>
            );
        } else {
            content = (
                <div id="loginContainer">
                    <div id="loginScreen">
                        <LoginForm
                            api={this.api}
                            onLogin={this.onLogin.bind(this)}
                        />
                    </div>
                </div>
            );
        }

        return <div id="controlFrame">{content}</div>;
    }
}

export interface ControlState extends SceneObjectState {}

export class Control extends SceneObjectBase<ControlState> {
    static Component = ControlImpl;

    constructor(state = {}) {
        super(state);
    }
}
