import React, { Component, ReactNode } from "react";
import { Switch } from "./WaterApi";

type PowerSwitchProps = {
    switches: Switch[];
    onClick: (id: number) => void;
};

type PowerSwitchState = {
    hovered: boolean[];
};

// XXX: separate rendering from control logic structure
// TODO: optimize out translate and matrix and so
// TODO: exact positions

class PowerSwitch extends Component<PowerSwitchProps, PowerSwitchState> {
    static xdist: number = 15.875;

    constructor(props: PowerSwitchProps) {
        super(props);
        this.state = {
            hovered: Array(this.props.switches.length),
        };
    }

    defs(): ReactNode {
        return (
            <defs>
                {/* device box */}
                <filter
                    id="filterBoxShadow"
                    x="-0.038667062"
                    y="-0.047591778"
                    width="1.0773341"
                    height="1.0951836"
                >
                    <feGaussianBlur stdDeviation="1.1855987" />
                </filter>

                <filter
                    id="filterBoxSpec"
                    x="-0.027653968"
                    y="-0.18163683"
                    width="1.0553079"
                    height="1.3632737"
                >
                    <feGaussianBlur stdDeviation="0.83572366" />
                </filter>

                <radialGradient
                    id="radialGradientBox"
                    cx="49.967976"
                    cy="51.476395"
                    fx="49.967976"
                    fy="51.476395"
                    r="36.794067"
                    gradientTransform={
                        "matrix(1.7312234 -0.16899547 " +
                        "0.07155189 0.73299184 -41.203679 24.910165)"
                    }
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#dfdfdf" offset={0} />
                    <stop stopColor="#f4f4f4" offset={1} />
                </radialGradient>

                <linearGradient
                    id="linearGradientBoxBorder"
                    x1="55.550968"
                    y1="38.297565"
                    x2="61.956512"
                    y2="82.887947"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#e5e5e5" offset={0} />
                    <stop stopColor="#bfbfbf" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientBoxSpec1"
                    x1="131.91005"
                    y1="30.32"
                    x2="131.51784"
                    y2="75.36"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#ffffff" stopOpacity={0} offset={0} />
                    <stop stopColor="#ffffff" stopOpacity={1} offset="0.5" />
                    <stop stopColor="#ffffff" stopOpacity={0} offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientBoxSpec2"
                    x1="50.176556"
                    y1="81.868889"
                    x2="50.187046"
                    y2="75.725929"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#ffffff" stopOpacity={1} offset={0} />
                    <stop stopColor="#ffffff" stopOpacity={0} offset={1} />
                </linearGradient>

                {/* indicators */}
                <filter
                    id="filterHexBlur"
                    x="-0.018941928"
                    y="-0.019432102"
                    width="1.0378839"
                    height="1.0388642"
                >
                    <feGaussianBlur stdDeviation="0.31663494" />
                </filter>

                <filter
                    id="filterIndicatorShadow"
                    x="-0.047999999"
                    y="-0.047999999"
                    width="1.096"
                    height="1.096"
                >
                    <feGaussianBlur stdDeviation="1.1680378" />
                </filter>

                <radialGradient
                    id="radialGradientLedHex"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform={
                        "matrix(0.04223023,-0.02356009,0.02626614," +
                        "0.04708075,27.09993,72.454839)"
                    }
                    cx="310.44345"
                    cy="-285.63126"
                    fx="310.44345"
                    fy="-285.63126"
                    r="483.97446"
                >
                    <stop stopColor="#000000" stopOpacity={1} offset={0} />
                    <stop stopColor="#000000" stopOpacity={0} offset={1} />
                </radialGradient>

                <radialGradient
                    id="radialGradientLedGreen"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="matrix(4.552356,0,0,4.5523492,-7.4437071,-1261.42)"
                    cx="8.8198948"
                    cy="288.44736"
                    fx="8.8198948"
                    fy="288.44736"
                    r="6.4144688"
                >
                    <stop stopColor="#5cff69" offset={0} />
                    <stop stopColor="#00e113" offset="0.7570765" />
                    <stop stopColor="#007109" offset={1} />
                </radialGradient>

                <radialGradient
                    id="radialGradientLedRed"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="matrix(4.552356,0,0,4.5523492,-7.4437071,-1261.42)"
                    cx="8.8198948"
                    cy="288.44736"
                    fx="8.8198948"
                    fy="288.44736"
                    r="6.4144688"
                >
                    <stop stopColor="#ff5c5c" offset={0} />
                    <stop stopColor="#e10200" offset="0.7570765" />
                    <stop stopColor="#710000" offset={1} />
                </radialGradient>

                <linearGradient
                    id="linearGradientLedSpec"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="matrix(0.67709918,0,0,1.827381,14.98238,-493.38795)"
                    x1="14.566191"
                    y1="252.09871"
                    x2="53.988441"
                    y2="252.09871"
                >
                    <stop stopColor="#ffffff" stopOpacity={1} offset={0} />
                    <stop stopColor="#ffffff" stopOpacity={0} offset={1} />
                </linearGradient>

                {/* switch box */}
                <filter
                    id="filterSwitchBoxShadow"
                    x="-0.0405"
                    y="-0.02025"
                    width="1.081"
                    height="1.0405"
                >
                    <feGaussianBlur stdDeviation="0.421875" />
                </filter>

                <linearGradient
                    id="linearGradientSwitchBox"
                    x1="50.434643"
                    y1="26.790728"
                    x2="50.655758"
                    y2="76.548943"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="translate(0,0.52916829)"
                >
                    <stop stopColor="#fafafa" offset={0} />
                    <stop stopColor="#d9d9d9" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientSwitchBoxBorder"
                    x1="46.69791"
                    y1="30.404102"
                    x2="65.966881"
                    y2="74.085449"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="translate(0,0.52916829)"
                >
                    <stop stopColor="#ffffff" offset={0} />
                    <stop stopColor="#cccccc" offset={1} />
                </linearGradient>

                {/* switches */}
                <linearGradient
                    id="linearGradientSwitchInner"
                    x1="34.104649"
                    y1="40.505234"
                    x2="34.012547"
                    y2="65.104218"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#000000" offset={0} />
                    <stop stopColor="#404040" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientSwitchInnerBorder"
                    x1="29.234196"
                    y1="39.911938"
                    x2="39.30888"
                    y2="65.397926"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#999999" offset={0} />
                    <stop stopColor="#cccccc" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientSwitchOffEdge"
                    gradientUnits="userSpaceOnUse"
                    x1="38.803955"
                    y1="60.735035"
                    x2="29.143486"
                    y2="60.742825"
                    gradientTransform="translate(-0.1130867,0.18559265)"
                >
                    <stop stopColor="#8c8c8c" offset={0} />
                    <stop stopColor="#cccccc" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientSwitchOnEdge"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="translate(-0.1130867,0.18559265)"
                    x1="38.803955"
                    y1="60.735035"
                    x2="29.143486"
                    y2="60.742825"
                >
                    <stop stopColor="#e5e5e5" offset={0} />
                    <stop stopColor="#ffffff" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientSwitchOffMain"
                    x1="33.992939"
                    y1="41.960346"
                    x2="34.104553"
                    y2="63.705963"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="translate(-0.1130867,0.18559265)"
                >
                    <stop stopColor="#cccccc" offset={0} />
                    <stop stopColor="#d9d9d9" offset={0.5} />
                    <stop stopColor="#e5e5e5" offset={0.51682985} />
                    <stop stopColor="#ffffff" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientSwitchOnMain"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform="translate(-0.1130867,2.3022594)"
                    x1="33.992939"
                    y1="41.960346"
                    x2="34.104553"
                    y2="63.705963"
                >
                    <stop stopColor="#bfbfbf" offset={0} />
                    <stop stopColor="#d9d9d9" offset={0.41512719} />
                    <stop stopColor="#e5e5e5" offset={0.42999968} />
                    <stop stopColor="#ffffff" offset={1} />
                </linearGradient>

                <radialGradient
                    id="radialGradientSwitchOffShadow"
                    cx="36.101601"
                    cy="61.580254"
                    fx="36.101601"
                    fy="61.580254"
                    r="7.2458353"
                    gradientTransform="matrix(1,0,0,1.5812984,0,-37.659668)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#000000" stopOpacity={0.5} offset={0} />
                    <stop stopColor="#000000" stopOpacity={0} offset={1} />
                </radialGradient>

                <radialGradient
                    id="radialGradientSwitchOnShadow"
                    gradientUnits="userSpaceOnUse"
                    gradientTransform={
                        "matrix(1.311707,8.4514818e-4,-0.00137921," +
                        "2.1405959,-11.440488,-81.156355)"
                    }
                    cx="36.916763"
                    cy="59.90873"
                    fx="36.916763"
                    fy="59.90873"
                    r="7.2458353"
                >
                    <stop stopColor="#000000" stopOpacity={0.5} offset={0} />
                    <stop stopColor="#000000" stopOpacity={0} offset={1} />
                </radialGradient>

                {/* buttons TODO: dedup with water switch*/}
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

                {/* labels */}
                <filter id="filterTextShadow" colorInterpolationFilters="sRGB">
                    <feGaussianBlur stdDeviation="0.3" />
                </filter>
            </defs>
        );
    }

    shadow(): ReactNode {
        const count: number = this.props.switches.length;
        return (
            <rect
                opacity="0.3"
                fill="#000000"
                fillOpacity="1"
                stroke="none"
                filter="url(#filterBoxShadow)"
                id="boxShadow"
                width={16 + 34 * count}
                height="59.788414"
                x="14.754096"
                y="24.3144"
                ry="8"
            />
        );
    }

    device_frame(): ReactNode {
        const count: number = this.props.switches.length;
        return (
            <g>
                {/* box */}
                <rect
                    fill="url(#radialGradientBox)"
                    fillOpacity="1"
                    stroke="url(#linearGradientBoxBorder)"
                    strokeWidth="1"
                    paintOrder="markers stroke fill"
                    //73.588135
                    width={16 + 34 * count}
                    height="61.375916"
                    x="13.695764"
                    y="21.668564"
                    ry="8"
                />
                {/* 2x specular */}
                <path
                    fill="none"
                    stroke="url(#linearGradientBoxSpec1)"
                    strokeWidth="0.8"
                    strokeOpacity="1"
                    d={
                        "m 15,30 " +
                        "v 45.5 " +
                        "z " +
                        `m ${13.5 + 34 * count},0 ` +
                        "v 45.5 " +
                        "z"
                    }
                />
                <path
                    fill="url(#linearGradientBoxSpec2)"
                    fillOpacity="1"
                    stroke="none"
                    filter="url(#filterBoxSpec)"
                    d={
                        "m 19,71.472746 " +
                        "v 3.042566 " +
                        "c 0,4.432 3.568,8 8,8 " +
                        `h ${-10 + 34 * count} ` +
                        "c 4.432,0 8,-3.568 8,-8 " +
                        "v -3.042566 " +
                        "z"
                    }
                />
            </g>
        );
    }

    led_indicator(enabled: boolean, on_off: boolean): ReactNode {
        const darken_layer = enabled ? null : (
            <ellipse
                cy="51.693031"
                cx="32.707592"
                opacity="0.5"
                fill="#000000"
                fillOpacity="1"
                stroke="none"
                rx="29.201"
                ry="29.200945"
            />
        );

        const orb_gradient = on_off
            ? "url(#radialGradientLedGreen)"
            : "url(#radialGradientLedRed)";
        const stroke_color = on_off ? "#005c08" : "#5c0000";
        const ypos = on_off ? 26.484871 : 64.584895;

        return (
            <g
                transform={`matrix(0.13412322 0 0 0.13412322 29.792455 ${ypos})`}
            >
                {/* shadow */}
                <circle
                    r="29.200945"
                    opacity="0.5"
                    fill="#111111"
                    fillOpacity="1"
                    stroke="none"
                    filter="url(#filterIndicatorShadow)"
                    cx="34.194111"
                    cy="52.684044"
                />
                {/* orb */}
                <circle
                    r="29.200945"
                    fill={orb_gradient}
                    fillOpacity="1"
                    stroke={stroke_color}
                    strokeWidth="1.20448"
                    strokeOpacity="1"
                    cx="32.707592"
                    cy="51.693031"
                />
                {/* specular */}
                <ellipse
                    fill="url(#linearGradientLedSpec)"
                    fillOpacity="1"
                    stroke="none"
                    strokeWidth="0.294309"
                    strokeOpacity="1"
                    cx="38.191536"
                    cy="-32.707584"
                    rx="13.346388"
                    ry="20.512936"
                    transform="rotate(90)"
                />
                {/* hex */}
                <path
                    fill="none"
                    stroke="url(#radialGradientLedHex)"
                    strokeWidth="0.974299"
                    filter="url(#filterHexBlur)"
                    d={
                        "m 39.29124,78.300601 " +
                        "v -7.602168 " +
                        "l 6.583696,-3.801074 6.583598,3.801074 " +
                        "v 7.602158 " +
                        "l -6.583598,3.801095 " +
                        "z " +
                        "M 45.874936,66.897359 " +
                        "V 59.29519 " +
                        "l 6.583598,-3.801075 6.583697,3.801075 " +
                        "v 7.602169 " +
                        "l -6.583697,3.801083 " +
                        "z " +
                        "m 6.583598,-11.403244 " +
                        "v -7.602169 " +
                        "l 6.583697,-3.801083 6.583695,3.801083 " +
                        "v 7.60216 " +
                        "L 59.042231,59.29519 " +
                        "Z " +
                        "M 26.123945,78.300601 " +
                        "v -7.602168 " +
                        "l 6.583599,-3.801074 6.583696,3.801074 " +
                        "v 7.602158 " +
                        "l -6.583696,3.801095 " +
                        "z " +
                        "M 32.707544,66.897359 " +
                        "V 59.29519 " +
                        "l 6.583696,-3.801075 6.583696,3.801075 " +
                        "v 7.602169 " +
                        "L 39.29124,70.698433 " +
                        "Z " +
                        "M 39.29124,55.494115 " +
                        "v -7.602169 " +
                        "l 6.583696,-3.801083 6.583598,3.801083 " +
                        "v 7.60216 " +
                        "l -6.583598,3.801084 " +
                        "z " +
                        "m 6.583696,-11.403243 " +
                        "v -7.602169 " +
                        "l 6.583598,-3.801074 6.583697,3.801074 " +
                        "v 7.60216 " +
                        "l -6.583697,3.801083 " +
                        "z " +
                        "M 12.956553,78.300601 " +
                        "v -7.602168 " +
                        "l 6.583696,-3.801074 6.583696,3.801074 " +
                        "v 7.602158 " +
                        "l -6.583696,3.801095 " +
                        "z " +
                        "M 19.540249,66.897359 " +
                        "V 59.29519 " +
                        "l 6.583696,-3.801075 6.583599,3.801075 " +
                        "v 7.602169 " +
                        "l -6.583599,3.801074 " +
                        "z " +
                        "m 6.583696,-11.403244 " +
                        "v -7.602169 " +
                        "l 6.583599,-3.801083 6.583696,3.801083 " +
                        "v 7.60216 " +
                        "l -6.583696,3.801084 " +
                        "z " +
                        "m 6.583599,-11.403252 " +
                        "v -7.60216 " +
                        "l 6.583696,-3.801074 6.583696,3.801074 " +
                        "v 7.60216 " +
                        "L 39.29124,47.891946 " +
                        "Z " +
                        "M 39.29124,32.687629 " +
                        "V 25.08546 " +
                        "l 6.583696,-3.801084 6.583598,3.801084 " +
                        "v 7.602159 " +
                        "l -6.583598,3.801084 " +
                        "z " +
                        "M 6.3729538,66.897359 " +
                        "V 59.29519 " +
                        "l 6.5835992,-3.801075 6.583696,3.801075 " +
                        "v 7.602169 " +
                        "l -6.583696,3.801083 " +
                        "z " +
                        "M 12.956553,55.494115 " +
                        "v -7.602169 " +
                        "l 6.583696,-3.801083 6.583696,3.801083 " +
                        "v 7.60216 " +
                        "l -6.583696,3.801084 " +
                        "z " +
                        "m 6.583696,-11.403252 " +
                        "v -7.60216 " +
                        "l 6.583696,-3.801074 6.583599,3.801074 " +
                        "v 7.60216 " +
                        "l -6.583599,3.801083 " +
                        "z " +
                        "M 26.123945,32.687629 " +
                        "V 25.08546 " +
                        "l 6.583599,-3.801084 6.583696,3.801084 " +
                        "v 7.602159 " +
                        "l -6.583696,3.801084 " +
                        "z " +
                        "M -0.21074159,55.494115 " +
                        "v -7.602169 " +
                        "l 6.58369539,-3.801083 6.5835992,3.801083 " +
                        "v 7.60216 " +
                        "L 6.3729538,59.29519 " +
                        "Z " +
                        "M 6.3729538,44.090863 " +
                        "v -7.60216 " +
                        "l 6.5835992,-3.801074 6.583696,3.801074 " +
                        "v 7.60216 " +
                        "l -6.583696,3.801083 " +
                        "z " +
                        "M 12.956553,32.687629 " +
                        "V 25.08546 " +
                        "l 6.583696,-3.801084 6.583696,3.801084 " +
                        "v 7.602159 " +
                        "l -6.583696,3.801084 " +
                        "z " +
                        "m 0,0 " +
                        "V 25.08546 " +
                        "l 6.583696,-3.801084 6.583696,3.801084 " +
                        "v 7.602159 " +
                        "l -6.583696,3.801084 " +
                        "z"
                    }
                    transform="matrix(1.0657329,0,0,1.0657329,-2.1499656,-3.3979328)"
                />
                {darken_layer}
            </g>
        );
    }

    switches(): ReactNode {
        let count = this.props.switches.length;
        let switches = Array(count);
        for (let i = 0; i < count; i++) {
            switches[i] = this.switch_ctrl(i);
        }
        return switches;
    }

    switch_frame(pos: number): ReactNode {
        return (
            <g transform="translate(-22)">
                {/* shadow */}
                <rect
                    opacity="0.5"
                    fill="#000000"
                    fillOpacity="1"
                    stroke="none"
                    filter="url(#filterSwitchBoxShadow)"
                    width="25"
                    height="50"
                    x="44.208469"
                    y="27.997284"
                    rx="3"
                />
                {/* box */}
                <rect
                    opacity="1"
                    fill="url(#linearGradientSwitchBox)"
                    fillOpacity="1"
                    stroke="url(#linearGradientSwitchBoxBorder)"
                    strokeWidth="1"
                    paintOrder="markers stroke fill"
                    width="25"
                    height="50"
                    x="43.679302"
                    y="27.468119"
                    rx="3"
                />
            </g>
        );
    }

    switch_ctrl(pos: number): ReactNode {
        let switch_ = null;

        const switch_frame = this.switch_frame(pos);
        const switch_button = this.button(pos, this.state.hovered[pos]);

        if (this.props.switches[pos].open === true)
            switch_ = this.open_switch(pos);
        else switch_ = this.closed_switch(pos);

        return (
            <g
                key={"switch" + pos}
                onClick={this.onClick.bind(this, pos)}
                onMouseEnter={this.onMouseEnter.bind(this, pos)}
                onMouseLeave={this.onMouseLeave.bind(this, pos)}
                transform={`translate(${3 + 35 * pos})`}
            >
                {switch_frame}
                {switch_button}
                {switch_}
            </g>
        );
    }

    open_switch(pos: number): ReactNode {
        // TODO: dedup with closed switch
        return (
            <g>
                {/* inset border */}
                <rect
                    opacity="1"
                    fill="url(#linearGradientSwitchInner)"
                    fillOpacity="1"
                    stroke="url(#linearGradientSwitchInnerBorder)"
                    strokeWidth="1"
                    paintOrder="markers stroke fill"
                    width="12.5"
                    height="25"
                    x="27.706167"
                    y="40.578976"
                    rx="2"
                    ry="2"
                />
                {/* top/bottom side */}
                <rect
                    fill="url(#linearGradientSwitchOffEdge)"
                    fillOpacity="1"
                    stroke="none"
                    width="10"
                    height="22"
                    x="28.956167"
                    y="42.078976"
                    rx="2.0000012"
                    ry="2.0000012"
                />
                {/* shadow */}
                <path
                    fill="url(#radialGradientSwitchOffShadow)"
                    fillOpacity="1"
                    stroke="none"
                    d={
                        "M 39.115434,53.327605 43.65326,68.790359 " +
                        "35.234787,76.243261 29.161589,60.925528 " +
                        "Z"
                    }
                />
                {/* switch */}
                <rect
                    fill="url(#linearGradientSwitchOffMain)"
                    fillOpacity="1"
                    stroke="none"
                    width="9.9999981"
                    height="19.883331"
                    x="28.956167"
                    y="42.078976"
                    rx="2.0000012"
                    ry="2.0000012"
                />
                {this.led_indicator(true, true)}
                {this.led_indicator(false, false)}
            </g>
        );
    }

    closed_switch(pos: number): ReactNode {
        // TODO: dedup with closed switch
        return (
            <g>
                {/* inset border */}
                <rect
                    opacity="1"
                    fill="url(#linearGradientSwitchInner)"
                    fillOpacity="1"
                    stroke="url(#linearGradientSwitchInnerBorder)"
                    strokeWidth="1"
                    paintOrder="markers stroke fill"
                    width="12.5"
                    height="25"
                    x="27.706167"
                    y="40.578976"
                    rx="2"
                    ry="2"
                />
                {/* shadow */}
                <path
                    fill="url(#radialGradientSwitchOnShadow)"
                    fillOpacity="1"
                    stroke="none"
                    d={
                        "M 39.115434,44.331766 43.65326,59.79452 " +
                        "35.234787,67.247422 28.96774,53.943074 " +
                        "29.002985,44.262928 " +
                        "Z"
                    }
                />
                {/* top/bottom side */}
                <rect
                    fill="url(#linearGradientSwitchOnEdge)"
                    fillOpacity="1"
                    stroke="none"
                    width="10"
                    height="22"
                    x="28.956167"
                    y="42.078976"
                    rx="2.0000012"
                    ry="2.0000012"
                />
                {/* switch */}
                <rect
                    fill="url(#linearGradientSwitchOnMain)"
                    fillOpacity="1"
                    stroke="none"
                    width="9.9999981"
                    height="19.883331"
                    x="28.956167"
                    y="44.195644"
                    rx="2.0000012"
                    ry="2.0000012"
                />
                {this.led_indicator(false, true)}
                {this.led_indicator(true, false)}
            </g>
        );
    }

    // TODO: dedup these buttons somehow between water and power
    button(pos: number, hovered: boolean) {
        const stroke_color = hovered ? "#666" : "#000";
        const fill_gradient = hovered ? "Button" : "ButtonHovered";
        return (
            <g
                transform={`matrix(2.9 0 0 3.4 ${PowerSwitch.xdist - 58.3} ${
                    -13.758 - 56.3
                })`}
            >
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

    // TODO: dedup with water switch
    label(pos: number, text: string): ReactNode {
        return (
            <g transform=
                {`matrix(2 0 0 2 ${-35 + 2.22 * PowerSwitch.xdist * pos} -90)`}
            >
                {/* text shadow */}
                <text
                    transform="rotate(-90)"
                    x="-88"
                    y="36.5"
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
                    x="-88"
                    y="36.5"
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
        if(this.props.switches.length === 0)
            return 0;
        return this.props.switches.reduce((a: Switch, b: Switch) => {
            return a.name.length > b.name.length ? a : b;
        }).name.length;
    }

    render() {
        const count: number = this.props.switches.length;
        return (
            <svg viewBox={`8 20 ${40 + 30*count} ${65 + 15*this.label_len()}`}>
                {this.defs()}
                {this.shadow()}
                {this.device_frame()}
                {this.switches()}
                {this.labels()}
            </svg>
        );
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
        this.props.onClick(channel);
    };
}

export default PowerSwitch;
