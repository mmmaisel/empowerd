import React, { Component, ReactNode } from "react";
import { Switch } from "./EmpowerdApi";

type SwitchWidgetProps = {
    switches: Switch[];
    onClick: (id: number) => void;
};

type SwitchWidgetState = {
    hovered: boolean[];
};

export class SvgMatrix {
    matrix: [number, number, number, number, number, number];

    constructor(
        x1: number,
        x2: number,
        x3: number,
        y1: number,
        y2: number,
        y3: number
    ) {
        this.matrix = [x1, x2, x3, y1, y2, y3];
    }

    toSvg = (): string => {
        return (
            "matrix(" +
            `${this.matrix[0]} ${this.matrix[1]} ` +
            `${this.matrix[2]} ${this.matrix[3]} ` +
            `${this.matrix[4]} ${this.matrix[5]})`
        );
    };
}

abstract class SwitchWidget extends Component<
    SwitchWidgetProps,
    SwitchWidgetState
> {
    constructor(props: SwitchWidgetProps) {
        super(props);
        this.state = {
            hovered: Array(this.props.switches.length),
        };
    }

    abstract buttonMatrix(pos: number): SvgMatrix;
    abstract labelMatrix(pos: number): SvgMatrix;

    common_defs(): ReactNode {
        return (
            <React.Fragment>
                {/* buttons */}
                <linearGradient
                    id="linearGradientButton"
                    x1="26.458"
                    x2="26.458"
                    y1="28.325"
                    y2="43.642"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#fff" stopOpacity=".78431" offset={0} />
                    <stop
                        stopColor="#fff"
                        stopOpacity=".5098"
                        offset=".11997"
                    />
                    <stop stopColor="#fff" stopOpacity={0} offset=".29271" />
                    <stop stopOpacity={0} offset={1} />
                </linearGradient>
                <linearGradient
                    id="linearGradientButtonHovered"
                    x1="26.458"
                    x2="26.458"
                    y1="28.325"
                    y2="43.642"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#fff" stopOpacity=".78431" offset={0} />
                    <stop
                        stopColor="#fff"
                        stopOpacity=".8259"
                        offset=".11997"
                    />
                    <stop stopColor="#fff" stopOpacity=".2353" offset=".4" />
                    <stop stopColor="#fff" stopOpacity=".1353" offset={1} />
                </linearGradient>
                <linearGradient
                    id="linearGradientButtonPushed"
                    x1="26.458"
                    x2="26.458"
                    y1="28.325"
                    y2="43.642"
                    gradientTransform="translate(-52.917 -71.967)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#fff" stopOpacity=".39216" offset={0} />
                    <stop
                        stopColor="#fff"
                        stopOpacity=".23529"
                        offset=".11997"
                    />
                    <stop stopColor="#fff" stopOpacity={0} offset=".29271" />
                    <stop
                        stopColor="#fff"
                        stopOpacity=".051522"
                        offset=".70729"
                    />
                    <stop
                        stopColor="#fff"
                        stopOpacity=".5098"
                        offset=".88819"
                    />
                    <stop stopColor="#fff" stopOpacity=".70588" offset={1} />
                </linearGradient>
                <linearGradient
                    id="linearGradientButtonPushedStroke"
                    x1="21.975"
                    x2="30.692"
                    y1="28.325"
                    y2="39.158"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop offset={0} />
                    <stop stopOpacity={0} offset={1} />
                </linearGradient>

                {/* labels */}
                <filter id="filterTextShadow" colorInterpolationFilters="sRGB">
                    <feGaussianBlur stdDeviation="0.3" />
                </filter>
            </React.Fragment>
        );
    }

    button(pos: number, hovered: boolean) {
        const stroke_color = hovered ? "#666" : "#000";
        const fill_gradient = hovered ? "Button" : "ButtonHovered";
        return (
            <g transform={this.buttonMatrix(pos).toSvg()}>
                <rect
                    x="22.225"
                    y="28.575"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fillOpacity=".5082"
                    stroke={stroke_color}
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeOpacity=".5098"
                    strokeWidth=".5"
                />
                <rect
                    x="22.225"
                    y="28.575"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fill={`url(#linearGradient${fill_gradient})`}
                />
            </g>
        );
    }

    // TODO: use it correctly
    // TODO: add hover
    button_pushed(pos: number): ReactNode {
        return (
            <g transform={this.buttonMatrix(pos).toSvg()}>
                <rect
                    x="22.225"
                    y="28.575"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fillOpacity=".5082"
                    stroke="#000"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeOpacity=".5098"
                    strokeWidth=".5"
                />
                <rect
                    transform="scale(-1)"
                    x="-30.692"
                    y="-43.392"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fill="url(#linearGradientButtonPushed)"
                />
                <rect
                    x="22.225"
                    y="28.575"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fill="none"
                    stroke="url(#linearGradientButtonPushedStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeOpacity=".5098"
                    strokeWidth=".5"
                />
            </g>
        );
    }

    label(pos: number, text: string): ReactNode {
        return (
            <g transform={this.labelMatrix(pos).toSvg()}>
                {/* text shadow */}
                <text
                    transform="rotate(-90)"
                    x="-38"
                    y="28.5"
                    fontFamily="sans-serif"
                    fontSize="6px"
                    stroke="#FFFFFF"
                    filter="url(#filterTextShadow)"
                    pointerEvents="none"
                    strokeWidth=".5"
                    text-align="end"
                    textAnchor="end"
                >
                    {text}
                </text>
                {/* text itself */}
                <text
                    transform="rotate(-90)"
                    x="-38"
                    y="28.5"
                    fontFamily="sans-serif"
                    fontSize="6px"
                    pointerEvents="none"
                    strokeWidth=".25"
                    text-align="end"
                    textAnchor="end"
                >
                    {text}
                </text>
            </g>
        );
    }

    labels(): ReactNode[] {
        let count: number = this.props.switches.length;
        let labels: ReactNode[] = Array<ReactNode>(count);
        for (let i = 0; i < count; i++) {
            labels[i] = this.label(i, this.props.switches[i].name);
        }
        return labels;
    }

    label_len(): number {
        if (this.props.switches.length === 0) return 0;
        return this.props.switches.reduce((a: Switch, b: Switch) => {
            return a.name.length > b.name.length ? a : b;
        }).name.length;
    }

    onMouseEnter = (channel: number): void => {
        let hovered = this.state.hovered;
        hovered[channel] = true;
        this.setState({ hovered: hovered });
    };

    onMouseLeave = (channel: number): void => {
        let hovered = this.state.hovered;
        hovered[channel] = false;
        this.setState({ hovered: hovered });
    };

    onClick = (channel: number): void => {
        this.props.onClick(this.props.switches[channel].id);
    };
}

export default SwitchWidget;
