import React, { Component, ReactNode } from "react";
import WaterSwitch, { WaterSwitchConfig } from "./WaterSwitch";
import PowerSwitch from "./PowerSwitch";
import PoweroffTimerConfig, { NamedPoweroffTimer } from "./PoweroffTimerConfig";
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
    poweroff_timer_modal: NamedPoweroffTimer | null;
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switches: [],
            poweroff_timers: [],
            poweroff_timer_modal: null,
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
            return x.switchId === switch_.id;
        });

        if (timer === undefined) {
            console.log(`Could not find poweroff_timer with id '${id}'.`);
            return;
        }

        this.setState({
            poweroff_timer_modal: {
                timer,
                name: switch_.name,
            },
        });
    };

    onClosePoweroffTimerModal = (
        on_time: number | null,
        canceled: boolean
    ): void => {
        if (
            canceled ||
            on_time === null ||
            this.state.poweroff_timer_modal === null
        ) {
            this.setState({ poweroff_timer_modal: null });
            return;
        }

        let timer = this.state.poweroff_timer_modal.timer;

        this.props.api.setPoweroffTimer(
            timer.id,
            on_time,
            (response: PoweroffTimer) => {
                let timers = this.state.poweroff_timers;
                timers[timer.id].onTime = response.onTime;
                this.setState({
                    poweroff_timers: timers,
                    poweroff_timer_modal: null,
                });
            },
            (errors: GraphQlError[]) => {
                alert("Setting poweroff timer failed");
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
                acc[timer.switchId] = {
                    id: timer.id,
                    name: this.state.switches[timer.switchId].name,
                    on_time: timer.onTime,
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
                <PoweroffTimerConfig
                    timer={this.state.poweroff_timer_modal}
                    onClose={this.onClosePoweroffTimerModal}
                />
            </div>
        );
    }
}

export default Status;
