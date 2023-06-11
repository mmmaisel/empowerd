import React, { Component, ReactNode } from "react";
import SwitchItem from "./SwitchItem";
import { TriState } from "./EmpowerdApi";
import "./SwitchWidget.scss";

// TODO: dedup PowerSwitch/WaterSwitch
//
type PowerSwitchProps = {
    switches: Map<string, SwitchItem>;
    onClick: (key: string) => void;
};

class PowerSwitch extends Component<PowerSwitchProps, {}> {
    imgFromState(state: TriState): string {
        if (state === TriState.On) return "on";
        else if (state === TriState.Off) return "off";
        else return "auto";
    }

    render(): ReactNode {
        const count: number = this.props.switches.size;
        if (count === 0) return null;

        let buttons: ReactNode[] = Array<ReactNode>(count);
        let i = 0;
        for (const [key, sw] of this.props.switches) {
            const img = this.imgFromState(sw.state);

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
                            onClick={this.props.onClick.bind(this, key)}
                        >
                            <img
                                alt={`switch-${img}`}
                                src={`power-switch.switch-${img}.svg`}
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
