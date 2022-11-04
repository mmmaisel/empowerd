import React, { Component, ReactElement, ReactNode } from "react";
import CSS from "csstype";
import WaterApi from "./WaterApi.jsx";
import "./Widgets.scss";

enum LoginState {
    IDLE = 0,
    BUSY,
    FAILED,
}

type LoginFormProps = {
    api: WaterApi;
    onLogin: () => void;
};

type LoginFormState = {
    login_state: LoginState;
    username: string;
    password: string;
};

class LoginForm extends Component<LoginFormProps, LoginFormState> {
    constructor(props: LoginFormProps) {
        super(props);
        this.state = {
            login_state: LoginState.IDLE,
            username: "",
            password: "",
        };
    }

    onLogin = (): void => {
        this.setState({ login_state: LoginState.BUSY });
        this.props.api.login(
            this.state.username,
            this.state.password,
            (_response: any) => {
                this.props.onLogin();
                this.setState({ login_state: LoginState.IDLE });
            },
            (_error: any) => {
                this.setState({ login_state: LoginState.FAILED });
            }
        );
    };

    onKeyDown = (event: React.KeyboardEvent): void => {
        if (event.keyCode === 13) {
            this.onLogin();
        }
    };

    onUsernameChanged = (event: React.ChangeEvent<HTMLInputElement>): void => {
        this.setState({ username: event.target.value });
    };

    onPasswordChanged = (event: React.ChangeEvent<HTMLInputElement>): void => {
        this.setState({ password: event.target.value });
    };

    loginState(): ReactElement {
        if (this.state.login_state === LoginState.BUSY) {
            return <div>Logging in...</div>;
        } else if (this.state.login_state === LoginState.FAILED) {
            return <div>Login failed...</div>;
        } else {
            return <React.Fragment />;
        }
    }

    render(): ReactNode {
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
                <div className="dialogTitle">Please Login</div>
                <div className="dialogContent">
                    <div style={grid}>
                        <span style={labelCol}> Username: </span>
                        <input
                            style={inputCol}
                            type="text"
                            value={this.state.username}
                            placeholder="username"
                            onChange={this.onUsernameChanged}
                            onKeyDown={this.onKeyDown}
                        />
                        <span style={labelCol}> Password: </span>
                        <input
                            style={inputCol}
                            type="password"
                            value={this.state.password}
                            onChange={this.onPasswordChanged}
                            onKeyDown={this.onKeyDown}
                        />
                    </div>
                    {this.loginState()}
                    <button className="button" onClick={this.onLogin}>
                        Login
                    </button>
                </div>
            </div>
        );
    }
}

export default LoginForm;
