import React, { Component, ReactNode } from "react";
import { PowerSwitch, WaterSwitch } from "./SwitchWidget";
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
    poweroffTimers: Map<string, PoweroffTimer>;
    poweroffTimerModal: NamedPoweroffTimer | null;
};

class Status extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switchItems: new Map<string, SwitchItem>(),
            poweroffTimers: new Map<string, PoweroffTimer>(),
            poweroffTimerModal: null,
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
        let sw = switch_maybe_undef;

        let timer = this.state.poweroffTimers.get(sw.configHandle() || "");
        if (timer === undefined) {
            console.log(`Could not find poweroff_timer with id '${key}'.`);
            return;
        }

        this.setState({
            poweroffTimerModal: {
                timer,
                name: sw.name,
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
            this.state.poweroffTimerModal === null
        ) {
            this.setState({ poweroffTimerModal: null });
            return;
        }

        let timer = this.state.poweroffTimerModal.timer;

        this.props.api.setPoweroffTimer(
            timer.id,
            on_time,
            (response: PoweroffTimer) => {
                let timers = this.state.poweroffTimers;
                let key = `timer${response.id}`;
                let timer = timers.get(key);

                if (timer === undefined) {
                    console.log(`Timer object '${response.id}' does not exist`);
                    return;
                }

                timer.onTime = response.onTime;
                timers.set(key, timer);
                this.setState({
                    poweroffTimers: timers,
                    poweroffTimerModal: null,
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
                    poweroffTimers: timers,
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

        return (
            <div className="mainframe">
                <WaterSwitch
                    switches={valves}
                    onClick={this.onSwitch}
                    onConfigure={this.onConfigureTimer}
                />
                <PowerSwitch
                    switches={switches}
                    onClick={this.onSwitch}
                    onConfigure={(key: string): void => {}}
                />
                <PoweroffTimerConfig
                    timer={this.state.poweroffTimerModal}
                    onClose={this.onClosePoweroffTimerModal}
                />
            </div>
        );
    }
}

export default Status;
