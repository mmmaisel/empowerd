import React, { Component, ReactNode } from "react";
import { PowerSwitch, WaterSwitch } from "./SwitchWidget";
import { SwitchItem, SwitchItemFactory } from "./SwitchItem";
import { EmpowerdApi, Appliance, GraphQlError, Switch } from "./EmpowerdApi";

type StatusProps = {
    api: EmpowerdApi;
};

type StatusState = {
    switchItems: Map<string, SwitchItem>;
};

export class SwitchesPanel extends Component<StatusProps, StatusState> {
    constructor(props: StatusProps) {
        super(props);
        this.state = {
            switchItems: new Map<string, SwitchItem>(),
        };
    }

    public onSwitch(key: string): void {
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
    }

    private cloneSwitchItems(): Map<string, SwitchItem> {
        let clone = new Map<string, SwitchItem>();
        for (const [k, v] of this.state.switchItems) {
            clone.set(k, v.clone());
        }
        return clone;
    }

    private loadAppliances(items: Map<string, SwitchItem>): void {
        this.props.api.appliances(
            (response: Appliance[]) => {
                for (const [k, _v] of items) {
                    if (k.startsWith("appliance")) items.delete(k);
                }

                for (const appliance of response) {
                    let item = SwitchItemFactory.fromAppliance(appliance);
                    items.set(item.key(), item);
                }

                this.setState({ switchItems: items });
                this.loadSwitches(items);
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
            }
        );
    }

    private loadSwitches(items: Map<string, SwitchItem>): void {
        this.props.api.switches(
            (response: Switch[]) => {
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
    }

    public componentDidMount(): void {
        this.loadAppliances(this.cloneSwitchItems());
    }

    public render(): ReactNode {
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
            <div id="switchesContainer">
                <WaterSwitch
                    switches={valves}
                    onClick={this.onSwitch.bind(this)}
                    onConfigure={(key: string): void => {}}
                />
                <PowerSwitch
                    switches={switches}
                    onClick={this.onSwitch.bind(this)}
                    onConfigure={(key: string): void => {}}
                />
            </div>
        );
    }
}
