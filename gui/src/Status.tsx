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
    switches: Map<string, Switch>;
    poweroff_timers: Map<string, PoweroffTimer>;
    poweroff_timer_modal: NamedPoweroffTimer | null;
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switches: new Map<string, Switch>(),
            poweroff_timers: new Map<string, PoweroffTimer>(),
            poweroff_timer_modal: null,
        };
    }

    onSwitch = (key: string): void => {
        let switches = this.state.switches;
        let switch_ = switches.get(key);

        if (switch_ === undefined) {
            console.log(`Could not find switch with id '${key}'.`);
            return;
        }

        this.props.api.setSwitch(
            switch_.id,
            !switch_.open,
            (response: Switch) => {
                let switches = this.state.switches;
                let key = `switch${response.id}`;

                let switch_ = switches.get(key);
                if (switch_ === undefined) {
                    console.log(`Switch object '${key}' does not exist`);
                    return;
                }

                switch_.open = response.open;
                switches.set(key, switch_);
                this.setState({ switches: switches });
            },
            (errors: GraphQlError[]) => {
                alert("Setting switch failed");
                console.log(errors);
            }
        );
    };

    onConfigureTimer = (key: string): void => {
        let switches = this.state.switches;
        let switch_maybe_undef = switches.get(key);

        if (switch_maybe_undef === undefined) {
            console.log(`Could not find switch with id '${key}'.`);
            return;
        }

        // Stupid Typescript does not recognize the guard above!
        let switch_ = switch_maybe_undef;

        let timers = this.state.poweroff_timers;
        let timer = null;
        for (const [_, x] of timers) {
            if (x.switchId === switch_.id) {
                timer = x;
                break;
            }
        }

        if (timer === null) {
            console.log(`Could not find poweroff_timer with id '${key}'.`);
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
                let key = `timer${response.id}`;
                let timer = timers.get(key);

                if (timer === undefined) {
                    console.log(`Timer object '${response.id}' does not exist`);
                    return;
                }

                timer.onTime = response.onTime;
                timers.set(key, timer);
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

    buildSwitchMap(switches: Switch[]): Map<string, Switch> {
        let result = new Map<string, Switch>();

        for (const switch_ of switches)
            result.set(`switch${switch_.id}`, switch_);

        return result;
    }

    buildTimerMap(timers: PoweroffTimer[]): Map<string, PoweroffTimer> {
        let result = new Map<string, PoweroffTimer>();

        for (const timer of timers)
            result.set(`poweroffTimer${timer.id}`, timer);

        return result;
    }

    componentDidMount(): void {
        this.props.api.switches(
            (response: Switch[]) => {
                this.setState({ switches: this.buildSwitchMap(response) });
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
            }
        );

        this.props.api.poweroffTimers(
            (response: PoweroffTimer[]) => {
                this.setState({
                    poweroff_timers: this.buildTimerMap(response),
                });
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
            }
        );
    }

    render(): ReactNode {
        // TODO: server time, manual trigger, next event
        // XXX: split after n items into another Switch widget

        let valves: Map<string, Switch> = new Map(
            [...this.state.switches].filter(([_, x]) => {
                return x.icon === "Valve";
            })
        );
        let switches: Map<string, Switch> = new Map(
            [...this.state.switches].filter(([_, x]) => {
                return x.icon === "Power";
            })
        );
        let configurations: Map<string, WaterSwitchConfig> = new Map(
            [...this.state.poweroff_timers].reduce((acc, [_key, timer], i) => {
                let switch_ = this.state.switches.get(
                    `switch${timer.switchId}`
                );
                if (switch_ === undefined) {
                    console.log("Missing switch for poweroffTimer");
                    return acc;
                }

                acc.push([
                    `switch${switch_.id}`,
                    {
                        id: timer.id,
                        name: switch_.name,
                        on_time: timer.onTime,
                    },
                ]);
                return acc;
            }, new Array<[string, WaterSwitchConfig]>())
        );

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
