import React, { Component, ReactNode } from "react";
import WaterSwitch, { WaterSwitchConfig } from "./WaterSwitch";
import PowerSwitch from "./PowerSwitch";
import PoweroffTimerConfig, { NamedPoweroffTimer } from "./PoweroffTimerConfig";
import SwitchItem, { SwitchItemFactory } from "./SwitchItem";
import EmpowerdApi, {
    Appliance,
    GraphQlError,
    PoweroffTimer,
    Switch,
} from "./EmpowerdApi";

// TODO: use React.fragment everywhere where possible

type StatusProps = {
    api: EmpowerdApi;
};

type StatusState = {
    switchItems: Map<string, SwitchItem>;
    poweroff_timers: Map<string, PoweroffTimer>;
    poweroff_timer_modal: NamedPoweroffTimer | null;
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switchItems: new Map<string, SwitchItem>(),
            poweroff_timers: new Map<string, PoweroffTimer>(),
            poweroff_timer_modal: null,
        };
    }

    onSwitch = (key: string): void => {
        let sw = this.state.switchItems.get(key);

        if (sw === undefined) {
            console.log(`Could not find switch with id ${key}.`);
            return;
        }

        let new_sw = sw.toggle();
        new_sw.save(
            this.props.api,
            (item: SwitchItem) => {
                let items = this.state.switchItems;
                items.set(item.key(), item);
                this.setState({ switchItems: items });
            },
            (error: string) => {
                alert(error);
            }
        );
    };

    onConfigureTimer = (key: string): void => {
        let switch_maybe_undef = this.state.switchItems.get(key);

        if (switch_maybe_undef === undefined) {
            console.log(`Could not find switch with id '${key}'.`);
            return;
        }

        // Stupid Typescript does not recognize the guard above!
        let switch_ = switch_maybe_undef;

        let timers = this.state.poweroff_timers;
        let timer = null;
        for (const [_k, x] of timers) {
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

    cloneSwitchItems(): Map<string, SwitchItem> {
        let clone = new Map<string, SwitchItem>();
        for (const [k, v] of this.state.switchItems) {
            clone.set(k, v.clone());
        }
        return clone;
    }

    componentDidMount(): void {
        this.props.api.appliances(
            (response: Appliance[]) => {
                let items = this.cloneSwitchItems();
                for (const [k, _v] of items) {
                    if (k.startsWith("appliance")) items.delete(k);
                }

                for (const appliance of response) {
                    let item = SwitchItemFactory.fromAppliance(appliance);
                    items.set(item.key(), item);
                }

                this.setState({ switchItems: items });
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
            }
        );
        this.props.api.switches(
            (response: Switch[]) => {
                let items = this.cloneSwitchItems();
                for (const [k, _v] of items) {
                    if (k.startsWith("switch")) items.delete(k);
                }

                for (const sw of response) {
                    let item = SwitchItemFactory.fromGpioSwitch(sw);
                    items.set(item.key(), item);
                }

                this.setState({ switchItems: items });
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
            }
        );

        this.props.api.poweroffTimers(
            (response: PoweroffTimer[]) => {
                let timers = new Map<string, PoweroffTimer>();

                for (const timer of response) {
                    timers.set(`poweroffTimer${timer.id}`, timer);
                }

                this.setState({
                    poweroff_timers: timers,
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

        let valves: Map<string, SwitchItem> = new Map(
            [...this.state.switchItems].filter(([_key, item]) => {
                return item.icon === "Valve";
            })
        );
        let switches: Map<string, SwitchItem> = new Map(
            [...this.state.switchItems].filter(([_key, item]) => {
                return item.icon === "Power";
            })
        );
        let configurations: Map<string, WaterSwitchConfig> = new Map(
            [...this.state.poweroff_timers].reduce((acc, [_key, timer], i) => {
                let sw = this.state.switchItems.get(`switch${timer.switchId}`);
                if (sw === undefined) {
                    console.log(
                        `Missing switch for poweroffTimer ${timer.switchId}.`
                    );
                    return acc;
                }

                acc.push([
                    sw.key(),
                    {
                        id: timer.id,
                        name: sw.name,
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
