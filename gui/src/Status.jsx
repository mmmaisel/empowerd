import React, { Component } from "react";
import WaterSwitch from "./WaterSwitch.jsx";

// TODO: use React.fragment everywhere where possible

class Status extends Component {
    constructor(props) {
        super(props);
        this.state = {
            switches: [],
        };
    }

    onSwitch = (channel) => {
        let id = this.state.switches[channel].id;
        let open = this.state.switches[channel].open;

        if (open === true) open = false;
        else open = true;

        this.props.api.setSwitch(
            id,
            open,
            (response) => {
                let switches = this.state.switches;
                switches[channel].open = response.setSwitch.open;
                this.setState({ switches: switches });
            },
            (error) => {
                alert("Setting switch failed");
                console.log(error);
            }
        );
    };

    // TODO: show if it is automatically activated
    // TODO: show remaining active time

    componentDidMount() {
        this.props.api.switches(
            (response) => {
                this.setState({ switches: response.switches });
            },
            (error) => {
                console.log(error);
            }
        );
    }

    render() {
        // TODO: server time, manual trigger, next event
        return (
            <div className="mainframe">
                <WaterSwitch
                    valves={this.state.switches}
                    onClick={this.onSwitch}
                />
            </div>
        );
    }
}

export default Status;
