import React, { Component, ReactElement, ReactNode } from "react";
import CSS from "csstype";
import { EmpowerdApi, GraphQlError } from "./EmpowerdApi";
import { t } from "../i18n";
import "./Control.scss";

enum LoginState {
    IDLE = 0,
    BUSY,
    FAILED,
}

type LoginFormProps = {
    api: EmpowerdApi;
    onLogin: () => void;
};

type LoginFormState = {
    login_state: LoginState;
    username: string;
    password: string;
};

// TODO: add message, e.g. "please log in" or "session timed out"
export class LoginForm extends Component<LoginFormProps, LoginFormState> {
    constructor(props: LoginFormProps) {
        super(props);
        this.state = {
            login_state: LoginState.IDLE,
            username: "",
            password: "",
        };
    }

    public onLogin(): void {
        this.setState({ login_state: LoginState.BUSY });
        this.props.api.login(
            this.state.username,
            this.state.password,
            () => {
                this.props.onLogin();
                this.setState({ login_state: LoginState.IDLE });
            },
            (errors: GraphQlError[]) => {
                this.setState({ login_state: LoginState.FAILED });
            }
        );
    }

    public onKeyDown(event: React.KeyboardEvent): void {
        if (event.key === "Enter") {
            this.onLogin();
        }
    }

    public onUsernameChanged(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ username: event.target.value });
    }

    public onPasswordChanged(event: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ password: event.target.value });
    }

    private loginState(): ReactElement {
        if (this.state.login_state === LoginState.BUSY) {
            return <div>{t("logging-in")}</div>;
        } else if (this.state.login_state === LoginState.FAILED) {
            return <div>{t("login-failed")}</div>;
        } else {
            return <React.Fragment />;
        }
    }

    public render(): ReactNode {
        let grid: CSS.Properties = {
            marginTop: "3px",
            display: "grid",
            gridGap: "3px",
            gridTemplateColumns: "[labels] auto [controls] 1fr",
            alignItems: "center",
            userSelect: "none",
        };
        let labelCol = {
            gridColumn: "labels",
        };
        let inputCol = {
            gridColumn: "controls",
        };
        return (
            <div className="dialogBorder">
                <div className="dialogTitle">{t("login-title")}</div>
                <div className="dialogContent">
                    <div style={grid}>
                        <span style={labelCol}>{t("username")}:</span>
                        <input
                            className="dialogInput"
                            style={inputCol}
                            type="text"
                            value={this.state.username}
                            placeholder={t("username")}
                            onChange={this.onUsernameChanged.bind(this)}
                            onKeyDown={this.onKeyDown.bind(this)}
                        />
                        <span style={labelCol}>{t("password")}:</span>
                        <input
                            className="dialogInput"
                            style={inputCol}
                            type="password"
                            value={this.state.password}
                            onChange={this.onPasswordChanged.bind(this)}
                            onKeyDown={this.onKeyDown.bind(this)}
                        />
                    </div>
                    {this.loginState()}
                    <button
                        className="dialogButton"
                        onClick={this.onLogin.bind(this)}
                    >
                        {t("login")}
                    </button>
                </div>
            </div>
        );
    }
}
