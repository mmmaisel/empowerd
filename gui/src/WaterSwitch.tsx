import React, { Component, ReactNode } from "react";
import { Switch } from "./EmpowerdApi";
import "./WaterSwitch.scss";

type WaterSwitchProps = {
    switches: Switch[];
    onClick: (id: number) => void;
};

class WaterSwitch extends Component<WaterSwitchProps, {}> {
    render(): ReactNode {
        const count: number = this.props.switches.length;

        let pipes: ReactNode[] = Array<ReactNode>(count);
        for (let i = 0; i < count; i++) {
            const img = this.props.switches[i].open === true ? "open" : "close";
            pipes[i] = (
                <React.Fragment>
                    <img
                        style={{ gridArea: `1/${i + 2}/3/${i + 2}` }}
                        alt=""
                        src={`water-switch.tile-${img}.svg`}
                    />
                    <div style={{ gridArea: `1/${i + 2}/1/${i + 2}` }}></div>
                    <div style={{ gridArea: `2/${i + 2}/2/${i + 2}` }}>
                        <div
                            className="btn"
                            style={{ marginBottom: "96px" }}
                            onClick={this.props.onClick.bind(this, i)}
                        >
                            <img
                                alt={`valve-${img}`}
                                src={`water-switch.valve-${img}.svg`}
                            />
                        </div>
                    </div>
                    <div
                        className="name"
                        style={{ gridArea: `3/${i + 2}/3/${i + 2}` }}
                    >
                        {this.props.switches[i].name}
                    </div>
                </React.Fragment>
            );
        }

        return (
            <div
                className="waterSwitch"
                style={{ gridTemplateColumns: `repeat(${count + 2}, 1fr)` }}
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
