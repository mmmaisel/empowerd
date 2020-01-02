import React, {Component} from 'react';
import './App.scss';

import Navbar from './Navbar.js';
import Config from './Config.js';
import Status from './Status.js';
import LoginForm from './LoginForm.js';

class App extends Component
{
    constructor(props)
    {
        super(props);
        this.items = ["Status", "Config", "Logout"];
        this.state =
        {
            logged_in: false,
            active_tab: this.items[0]
        };
    }

    onLogin = () =>
    {
        this.setState({ logged_in: true });
        // TODO: real login logic
    }

    onLogout = () =>
    {
        this.setState({ logged_in: false });
        // TODO: real logout logic
    }

    onTab = (which) =>
    {
        if(which === "Logout")
        {
            this.onLogout();
        }
        else
        {
            this.setState({ active_tab: which });
        }
    }

    render()
    {
        let content;
        if(this.state.logged_in)
        {
            if(this.state.active_tab === "Status")
            {
                content = <Status />
            }
            else if(this.state.active_tab === "Config")
            {
                content = <Config />
            }
            else
            {
                throw("Invalid active_tab");
            }
            return (
                <div>
                    <Navbar items={ this.items } onTab={ this.onTab }
                        active_tab={ this.state.active_tab } />
                    { content }
                </div>
            );
        }
        else
        {
            return (
                <div className="loginScreen">
                    <LoginForm onLogin={ this.onLogin }/>
                </div>
            );
        }
    }
}

export default App;
