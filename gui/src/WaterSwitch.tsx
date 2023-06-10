import React, { Component, ReactNode } from "react";
import { Switch } from "./EmpowerdApi";
import "./SwitchWidget.scss";

export type WaterSwitchConfig = {
    id: number;
    name: String;
    on_time: number;
};

type WaterSwitchProps = {
    switches: Switch[];
    onClick: (id: number) => void;
    onConfigure: (id: number) => void;
    configurations: (WaterSwitchConfig | null)[];
};

class WaterSwitch extends Component<WaterSwitchProps, {}> {
    render(): ReactNode {
        const count: number = this.props.switches.length;
        if (count === 0) return null;

        let pipes: ReactNode[] = Array<ReactNode>(count);
        for (let i = 0; i < count; ++i) {
            const switch_ = this.props.switches[i];
            const img = switch_.open === true ? "open" : "close";

            let config = null;
            if (this.props.configurations[i]) {
                config = (
                    <div style={{ gridArea: `1/${i + 2}/1/${i + 2}` }}>
                        <div
                            className="btn"
                            onClick={this.props.onConfigure.bind(
                                this,
                                switch_.id
                            )}
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
                    {config}
                    <div style={{ gridArea: `2/${i + 2}/2/${i + 2}` }}>
                        <div
                            className="btn"
                            onClick={this.props.onClick.bind(this, switch_.id)}
                        >
                            <img
                                alt={`valve-${img}`}
                                src={`water-switch.valve-${img}.svg`}
                            />
                        </div>
                    </div>
                    <div style={{ gridArea: `3/${i + 2}/3/${i + 2}` }}>
                        <div className="name">{switch_.name}</div>
                    </div>
                </React.Fragment>
            );
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
