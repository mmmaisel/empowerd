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
        let id = this.state.valves[channel].id;
        let open = this.state.valves[channel].open;

        if (open === true) open = false;
        else open = true;

        this.props.api.setValve(
            id,
            open,
            (response) => {
                let valves = this.state.valves;
                valves[channel].open = response.setValve.open;
                this.setState({ valves: valves });
            },
            (error) => {
                alert("Setting valve failed");
                console.log(error);
            }
        );
    };

    // TODO: show if it is automatically activated
    // TODO: show remaining active time

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
