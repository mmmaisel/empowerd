import React, { Component, ReactNode } from "react";
import WaterSwitch, { WaterSwitchConfig } from "./WaterSwitch";
import PowerSwitch from "./PowerSwitch";
import EmpowerdApi, {
    GraphQlError,
    PoweroffTimer,
    Switch,
} from "./EmpowerdApi";

// TODO: use React.fragment everywhere where possible

type StatusProps = {
    api: EmpowerdApi;
};

type StatusState = {
    switches: Switch[];
    poweroff_timers: PoweroffTimer[];
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switches: [],
            poweroff_timers: [],
        };
    }

    onSwitch = (id: number): void => {
        let switches = this.state.switches;
        let switch_ = switches.find((x: Switch) => {
            return x.id === id;
        });

        if (switch_ === undefined) {
            console.log(`Could not find switch with id '${id}'.`);
            return;
        }

        this.props.api.setSwitch(
            id,
            !switch_.open,
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

    onConfigureTimer = (id: number): void => {
        let switches = this.state.switches;
        let switch_maybe_undef = switches.find((x: Switch) => {
            return x.id === id;
        });

        if (switch_maybe_undef === undefined) {
            console.log(`Could not find switch with id '${id}'.`);
            return;
        }

        // Stupid Typescript does not recognize the guard above!
        let switch_ = switch_maybe_undef;

        let timers = this.state.poweroff_timers;
        let timer = timers.find((x: PoweroffTimer) => {
            return x.switch_id === switch_.id;
        });

        if (timer === undefined) {
            console.log(`Could not find poweroff_timer with id '${id}'.`);
            return;
        }
        alert(`Clicked configure ${timer.id}`);
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

        this.props.api.poweroffTimers(
            (response: PoweroffTimer[]) => {
                this.setState({ poweroff_timers: response });
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
        let configurations: (WaterSwitchConfig | null)[] =
            this.state.poweroff_timers.reduce((acc, timer, i) => {
                acc[timer.switch_id] = {
                    id: timer.id,
                    name: this.state.switches[timer.switch_id].name,
                    on_time: timer.on_time,
                };
                return acc;
            }, Array<WaterSwitchConfig | null>(valves.length).fill(null));

        return (
            <div className="mainframe">
                <WaterSwitch
                    switches={valves}
                    configurations={configurations}
                    onClick={this.onSwitch}
                    onConfigure={this.onConfigureTimer}
                />
                <PowerSwitch switches={switches} onClick={this.onSwitch} />
            </div>
        );
    }
}

export default Status;
