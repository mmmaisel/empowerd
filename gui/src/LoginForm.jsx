import React, { Component } from "react";
import "./Widgets.scss";

class LoginForm extends Component {
    static IDLE = 0;
    static BUSY = 1;
    static FAILED = 2;

    constructor(props) {
        super(props);
        this.state = {
            login_state: LoginForm.IDLE,
            username: "",
            password: "",
        };
    }

    onLogin = () => {
        this.setState({ login_state: LoginForm.BUSY });
        this.props.api.login(
            this.state.username,
            this.state.password,
            (response) => {
                this.props.onLogin();
                this.setState({ login_state: LoginForm.IDLE });
            },
            (error) => {
                this.setState({ login_state: LoginForm.FAILED });
            }
        );
    };

    onKeyDown = (event) => {
        if (event.keyCode === 13) {
            this.onLogin();
        }
    };

    onUsernameChanged = (event) => {
        this.setState({ username: event.target.value });
    };

    onPasswordChanged = (event) => {
        this.setState({ password: event.target.value });
    };

    loginState() {
        if (this.state.login_state === LoginForm.BUSY) {
            return <div>Logging in...</div>;
        } else if (this.state.login_state === LoginForm.FAILED) {
            return <div>Login failed...</div>;
        }
    }

    render() {
        let grid = {
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
