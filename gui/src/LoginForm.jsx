import React, {Component} from 'react';
import './Widgets.scss';

class LoginForm extends Component
{
    #token;

    constructor(props)
    {
        super(props);
        this.#token = "test"
        this.state =
        {
            username: "",
            password: "",
        };
    }

    login = () =>
    {
        this.props.onLogin();
        // TODO: ajax
        // TODO: pass logged in to parent
    }

    onUsernameChanged = (event) =>
    {
        this.setState({username: event.target.value});
    }

    onPasswordChanged = (event) =>
    {
        this.setState({password: event.target.value});
    }

    render()
    {
        let grid =
        {
            marginTop: "3px",
            display: "grid",
            gridGap: "3px",
            gridTemplateColumns: "[labels] auto [controls] 1fr",
            alignItems: "center",
            userSelect: "none"
        };
        let labelCol =
        {
            gridColumn: "labels",
        };
        let inputCol =
        {
            gridColumn: "controls"
        };
        return (
            <div className="dialogBorder">
                <div className="dialogTitle">
                    Please Login
                </div>
                <div className="dialogContent">
                    <div style={grid}>
                        <span style={labelCol}> Username: </span>
                        <input style={inputCol}
                            type="text" value={this.state.username}
                            placeholder="username"
                            onChange={this.onUsernameChanged}
                        />
                        <span style={labelCol}> Password: </span>
                        <input style={inputCol}
                            type="password" value={this.state.password}
                            onChange={this.onPasswordChanged}
                        />
                    </div>
                    <button className="button" onClick={this.login}>
                        Login
                    </button>
                </div>
            </div>
        );
    }
}

export default LoginForm;
