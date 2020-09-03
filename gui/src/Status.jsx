import React, { Component } from "react";
import WaterSwitch from "./WaterSwitch.jsx";

// TODO: use React.fragment everywhere where possible

class Status extends Component {
    constructor(props) {
        super(props);
        this.state = {
            valves: [],
        };
    }

    onValve = (channel) => {
        // TODO: post state change
        // TODO: then read state from server and update gui
        let valves = this.state.valves;
        if (valves[channel].open === true) valves[channel].open = false;
        else valves[channel].open = true;

        this.setState({ valves: valves });
    };

    // TODO: show if it is automatically activated
    // TODO: show remaining active time
    // TODO: show channel name

    componentDidMount() {
        this.props.api.valves(
            (response) => {
                this.setState({ valves: response.valves });
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
                    valves={this.state.valves}
                    onClick={this.onValve}
                />
            </div>
        );
    }
}

export default Status;
