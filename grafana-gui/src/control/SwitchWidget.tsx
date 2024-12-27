import React, { Component, ReactNode } from "react";
import { SwitchItem } from "./SwitchItem";
import { TriState } from "./EmpowerdApi";
import "./SwitchWidget.scss";

type SwitchWidgetProps = {
    switches: Map<string, SwitchItem>;
    onClick: (key: string) => void;
    onConfigure: (key: string) => void;
};

const PREFIX: string = "/public/plugins/empowerd/img/";

export abstract class SwitchWidget extends Component<SwitchWidgetProps, {}> {
    protected abstract graphics: string;
    protected abstract templateRows: string;

    private stateToImg(state: TriState): string {
        if (state === TriState.On) return "on";
        else if (state === TriState.Off) return "off";
        else return "auto";
    }

    public render(): ReactNode {
        const count: number = this.props.switches.size;
        if (count === 0) return null;

        let segments: ReactNode[] = Array<ReactNode>(count);
        let i = 0;
        for (const [key, sw] of this.props.switches) {
            const img = this.stateToImg(sw.state);

            let config_node = null;
            if (sw.configHandle !== null) {
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

            segments[i] = (
                <React.Fragment>
                    <img
                        style={{ gridArea: `1/${i + 2}/3/${i + 2}` }}
                        alt=""
                        src={`${PREFIX}${this.graphics}-switch.tile-${img}.svg`}
                    />
                    {config_node}
                    <div
                        className="btnContainer"
                        style={{ gridArea: `2/${i + 2}/2/${i + 2}` }}
                    >
                        <div
                            className="btn"
                            onClick={this.props.onClick.bind(this, key)}
                        >
                            <img
                                alt={`${PREFIX}${this.graphics}-switch-${img}`}
                                src={`${PREFIX}${this.graphics}-switch.${img}.svg`}
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
                    gridTemplateColumns: `repeat(${count + 2}, minmax(0, 1fr))`,
                    gridTemplateRows: this.templateRows,
                }}
            >
                <img
                    style={{ gridArea: "1/1/3/1" }}
                    alt=""
                    src={`${PREFIX}${this.graphics}-switch.tile-start.svg`}
                />
                <img
                    style={{ gridArea: `1/${count + 2}/3/${count + 2}` }}
                    alt=""
                    src={`${PREFIX}${this.graphics}-switch.tile-end.svg`}
                />
                {segments}
            </div>
        );
    }
}

export class PowerSwitch extends SwitchWidget {
    protected graphics: string = "power";
    protected templateRows: string = "0.35fr 0.55fr 1fr";
}

export class WaterSwitch extends SwitchWidget {
    protected graphics: string = "water";
    protected templateRows: string = "0.3fr 0.45fr 1fr";
}
