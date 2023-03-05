import React, { Component, ReactNode } from "react";
import WaterSwitch from "./WaterSwitch";
import PowerSwitch from "./PowerSwitch";
import EmpowerdApi, { GraphQlError, Switch } from "./EmpowerdApi";

// TODO: use React.fragment everywhere where possible

type StatusProps = {
    api: EmpowerdApi;
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

    onSwitch = (id: number): void => {
        let switches = this.state.switches;
        let switch_ = this.state.switches.find((x: Switch) => {
            return x.id === id;
        });

        if (switch_ === undefined) {
            console.log(`Could not find switch with id '${id}'.`);
            return;
        }

        this.props.api.setSwitch(
            id,
            switch_.open,
            (response: Switch) => {
                let switches = this.state.switches;
                switches[id].open = response.open;
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
        // XXX: split after n items into another Switch widget

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
