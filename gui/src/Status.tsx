import React, { Component, ReactNode } from "react";
import WaterSwitch from "./WaterSwitch";
import WaterApi, { GraphQlError, Switch } from "./WaterApi";

// TODO: use React.fragment everywhere where possible

type StatusProps = {
    api: WaterApi;
};

type StatusState = {
    switches: Switch[];
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switches: [],
        };
    }

    onSwitch = (channel: number): void => {
        let id: number = this.state.switches[channel].id;
        let open: boolean = this.state.switches[channel].open;

        this.props.api.setSwitch(
            id,
            open,
            (response: Switch) => {
                let switches = this.state.switches;
                switches[channel].open = response.open;
                this.setState({ switches: switches });
            },
            (errors: GraphQlError[]) => {
                alert("Setting switch failed");
                console.log(errors);
            }
        );
    };

    // TODO: show if it is automatically activated
    // TODO: show remaining active time

    componentDidMount(): void {
        this.props.api.switches(
            (response: Switch[]) => {
                this.setState({ switches: response });
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
            }
        );
    }

    render(): ReactNode {
        // TODO: server time, manual trigger, next event
        let valves: Switch[] = this.state.switches.filter((x) => {
            return x.icon === "Valve";
        });
        return (
            <div className="mainframe">
                <WaterSwitch valves={valves} onClick={this.onSwitch} />
            </div>
        );
    }
}

export default Status;
