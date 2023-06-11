import React, { Component, ReactNode } from "react";
import SwitchItem from "./SwitchItem";
import { TriState } from "./EmpowerdApi";
import "./SwitchWidget.scss";

export type WaterSwitchConfig = {
    id: number;
    name: String;
    on_time: number;
};

type WaterSwitchProps = {
    switches: Map<string, SwitchItem>;
    onClick: (key: string) => void;
    onConfigure: (key: string) => void;
    // TODO: replace configs with switchItem.configure()
    configurations: Map<string, WaterSwitchConfig>;
};

class WaterSwitch extends Component<WaterSwitchProps, {}> {
    imgFromState(state: TriState): string {
        if (state === TriState.On) return "open";
        else if (state === TriState.Off) return "close";
        else return "none";
    }

    render(): ReactNode {
        const count: number = this.props.switches.size;
        if (count === 0) return null;

        let pipes: ReactNode[] = Array<ReactNode>(count);
        let i = 0;
        for (const [key, sw] of this.props.switches) {
            const img = this.imgFromState(sw.state);

            let config_src = this.props.configurations.get(key);
            let config_node = null;

            if (config_src !== undefined) {
                config_node = (
                    <div style={{ gridArea: `1/${i + 2}/1/${i + 2}` }}>
                        <div
                            className="btn"
                            onClick={this.props.onConfigure.bind(this, key)}
                        >
                            <img alt="configure" src="config.svg" />
                        </div>
                    </div>
                );
            }

            pipes[i] = (
                <React.Fragment>
                    <img
                        style={{ gridArea: `1/${i + 2}/3/${i + 2}` }}
                        alt=""
                        src={`water-switch.tile-${img}.svg`}
                    />
                    {config_node}
                    <div style={{ gridArea: `2/${i + 2}/2/${i + 2}` }}>
                        <div
                            className="btn"
                            onClick={this.props.onClick.bind(this, key)}
                        >
                            <img
                                alt={`valve-${img}`}
                                src={`water-switch.valve-${img}.svg`}
                            />
                        </div>
                    </div>
                    <div style={{ gridArea: `3/${i + 2}/3/${i + 2}` }}>
                        <div className="name">{sw.name}</div>
                    </div>
                </React.Fragment>
            );
            ++i;
        }

        return (
            <div
                className="switchWidget"
                style={{
                    gridTemplateColumns: `repeat(${count + 2}, 1fr)`,
                    gridTemplateRows: "0.3fr 0.45fr 1fr",
                }}
            >
                <img
                    style={{ gridArea: "1/1/3/1" }}
                    alt=""
                    src="water-switch.tile-start.svg"
                />
                <img
                    style={{ gridArea: `1/${count + 2}/3/${count + 2}` }}
                    alt=""
                    src="water-switch.tile-end.svg"
                />
                {pipes}
            </div>
        );
    }
}

export default WaterSwitch;
