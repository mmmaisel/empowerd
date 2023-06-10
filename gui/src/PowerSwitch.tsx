import React, { Component, ReactNode } from "react";
import { Switch } from "./EmpowerdApi";
import "./SwitchWidget.scss";

type PowerSwitchProps = {
    switches: Switch[];
    onClick: (id: number) => void;
};

class PowerSwitch extends Component<PowerSwitchProps, {}> {
    render(): ReactNode {
        const count: number = this.props.switches.length;
        if (count === 0) return null;

        let buttons: ReactNode[] = Array<ReactNode>(count);
        for (let i = 0; i < count; ++i) {
            const switch_ = this.props.switches[i];
            const img = switch_.open === true ? "on" : "off";

            // TODO: add tri-state switch, keep two-state indicator
            buttons[i] = (
                <React.Fragment>
                    <img
                        style={{ gridArea: `1/${i + 2}/3/${i + 2}` }}
                        alt=""
                        src={`power-switch.tile-${img}.svg`}
                    />
                    <div style={{ gridArea: `2/${i + 2}/2/${i + 2}` }}>
                        <div
                            className="btn"
                            onClick={this.props.onClick.bind(this, switch_.id)}
                        >
                            <img
                                alt={`switch-${img}`}
                                src={`power-switch.switch-${img}.svg`}
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
                    gridTemplateRows: "0.35fr 0.55fr 1fr",
                }}
            >
                <img
                    style={{ gridArea: "1/1/3/1" }}
                    alt=""
                    src="power-switch.tile-start.svg"
                />
                <img
                    style={{ gridArea: `1/${count + 2}/3/${count + 2}` }}
                    alt=""
                    src="power-switch.tile-end.svg"
                />
                {buttons}
            </div>
        );
    }
}

export default PowerSwitch;
