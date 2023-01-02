import React, { Component, ReactNode } from "react";
import WaterSwitch from "./WaterSwitch";
import PowerSwitch from "./PowerSwitch";
import WaterApi, { GraphQlError, Switch } from "./WaterApi";

// TODO: use React.fragment everywhere where possible

type StatusProps = {
    api: WaterApi;
};

type StatusState = {
    switches: Switch[];
    test: Switch[];
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switches: [],
            test: [
                {
                    id: 1,
                    icon: "Power",
                    name: "on",
                    open: true
                },
                {
                    id: 2,
                    icon: "Power",
                    name: "off",
                    open: false
                },
                {
                    id: 3,
                    icon: "Power",
                    name: "bla",
                    open: false
                }
            ]
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

    onSwitchToggle = (channel: number): void => {
        let id: number = this.state.test[channel].id;
        let open: boolean = this.state.test[channel].open;

        let test = this.state.test;
        test[channel].open = !open;
        this.setState({ test: test });
    }

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
        let switches: Switch[] = this.state.switches.filter((x) => {
            return x.icon === "Power";
        });

        return (
            <div className="mainframe">
                <WaterSwitch switches={valves} onClick={this.onSwitch} />
                <PowerSwitch switches={switches} onClick={this.onSwitch} />
            </div>
        );
    }
}

export default Status;
