import React, { Component } from "react";

class WaterSwitch extends Component {
    static xdist = 15.875;

    constructor(props) {
        super(props);
        this.state = {
            hovered: Array(this.props.labels.length),
        };
    }

    defs() {
        return (
            <defs>
                <linearGradient id="linearGradientGoldFade">
                    <stop stopColor="#edcc84" offset={0} />
                    <stop stopColor="#edcc84" stopOpacity={0} offset={1} />
                </linearGradient>
                <linearGradient id="linearGradientValveMaterial">
                    <stop stopColor="#8e6815" offset={0} />
                    <stop stopColor="#e5b64e" offset=".34552" />
                    <stop stopColor="#d29a1f" offset=".66908" />
                    <stop stopColor="#956d16" offset={1} />
                </linearGradient>
                <linearGradient id="linearGradientValveTipMaterial">
                    <stop stopColor="#edcc84" offset={0} />
                    <stop stopColor="#e5b64e" offset=".34552" />
                    <stop stopColor="#d29a1f" offset=".66908" />
                    <stop stopColor="#956d16" offset={1} />
                </linearGradient>

                <linearGradient id="linearGradientPipe">
                    <stop stopColor="#4a4a4a" offset={0} />
                    <stop stopColor="#c3c3c3" offset=".38524" />
                    <stop stopColor="#989898" offset=".63557" />
                    <stop stopColor="#393939" offset={1} />
                </linearGradient>

                <linearGradient
                    id="linearGradientPipeH"
                    x1="33.867"
                    x2="33.867"
                    y1="21.167"
                    y2="31.75"
                    gradientTransform="translate(0 -13.758)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientPipe"
                />
                <linearGradient
                    id="linearGradientPipeV"
                    x1="25.4"
                    x2="31.75"
                    y1="43.392"
                    y2="43.392"
                    gradientTransform="translate(-2.1167 -6.35)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientPipe"
                />
                <linearGradient
                    id="linearGradientWindow"
                    x1="16.087"
                    x2="16.087"
                    y1="25.4"
                    y2="27.517"
                    gradientTransform="translate(-4.2333)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#8b8b8b" offset={0} />
                    <stop stopColor="#a8a8a8" offset=".5" />
                    <stop stopColor="#707070" offset={1} />
                </linearGradient>
                <filter
                    id="filterWaterDrop"
                    x="-.032272"
                    y="-.4262"
                    width="1.0645"
                    height="1.8524"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="0.040912515" />
                </filter>

                <linearGradient
                    id="linearGradientPipeCapL"
                    x1="3.175"
                    x2="3.175"
                    y1="20.108"
                    y2="32.808"
                    gradientTransform="translate(0 -13.758)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#373737" offset={0} />
                    <stop stopColor="#c3c3c3" offset=".33333" />
                    <stop stopColor="#989898" offset=".58333" />
                    <stop stopColor="#393939" offset={1} />
                </linearGradient>
                <linearGradient
                    id="linearGradientPipeCapR"
                    x1="64.558"
                    x2="64.558"
                    y1="22.225"
                    y2="30.692"
                    gradientTransform="translate(0 -13.758)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#4a4a4a" offset={0} />
                    <stop stopColor="#c3c3c3" offset=".25" />
                    <stop stopColor="#989898" offset=".5" />
                    <stop stopColor="#393939" offset={1} />
                </linearGradient>
                <filter
                    id="filterPipeShadow"
                    x="-.079"
                    y="-.24947"
                    width="1.158"
                    height="1.4989"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="2.0902082" />
                </filter>

                <linearGradient
                    id="linearGradientPipeVHoleStroke"
                    x1="23.283"
                    x2="29.633"
                    y1="44.45"
                    y2="44.45"
                    gradientTransform="translate(0 2.6458)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#343434" offset={0} />
                    <stop stopColor="#9e9e9e" offset=".38524" />
                    <stop stopColor="#7c7c7c" offset=".63557" />
                    <stop stopColor="#272727" offset={1} />
                </linearGradient>
                <radialGradient
                    id="radialGradientPipeVStroke"
                    cx="26.035"
                    cy="31.596"
                    r="3.175"
                    gradientTransform="matrix(2 1.0277e-6 -3.7681e-7 .73333 -26.458 4.1348)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#dfdfdf" offset={0} />
                    <stop stopColor="#707070" offset=".5" />
                    <stop stopColor="#2b2b2b" stopOpacity={0} offset={1} />
                </radialGradient>
                <linearGradient
                    id="linearGradientPipeVHole"
                    x1="23.177"
                    x2="29.739"
                    y1="44.45"
                    y2="44.45"
                    gradientTransform="translate(0 2.6458)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#1b1b1b" offset={0} />
                    <stop stopColor="#7c7c7c" offset=".46774" />
                    <stop stopColor="#b7b7b7" offset=".7258" />
                    <stop stopColor="#1c1c1c" offset={1} />
                </linearGradient>
                <linearGradient
                    id="linearGradientPipeVHoleShadow"
                    x1="26.458"
                    x2="26.458"
                    y1="41.91"
                    y2="46.99"
                    gradientTransform="translate(0 2.6458)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#040404" offset={0} />
                    <stop
                        stopColor="#1e1e1e"
                        stopOpacity=".028103"
                        offset={1}
                    />
                </linearGradient>

                <linearGradient
                    id="linearGradientWaterfall"
                    x1="26.458"
                    x2="26.458"
                    y1="42.333"
                    y2="57.15"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#195eae" offset={0} />
                    <stop stopColor="#1480e7" offset=".28571" />
                    <stop stopColor="#0897f4" offset=".57143" />
                    <stop stopColor="#64b4e9" offset={1} />
                </linearGradient>
                <radialGradient
                    id="radialGradientWaterfallSpecular"
                    cx="25.929"
                    cy="49.742"
                    r="1.5875"
                    gradientTransform="matrix(1 0 0 4.6667 2.4131 -182.28)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#e8f5fd" offset={0} />
                    <stop stopColor="#fff" stopOpacity={0} offset={1} />
                </radialGradient>
                <filter
                    id="filterWaterRippleBlur"
                    x="-.020571"
                    y="-.0288"
                    width="1.0411"
                    height="1.0576"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="0.12699999" />
                </filter>
                <filter
                    id="filterWaterPuddleBlur"
                    x="-.041143"
                    y="-.0576"
                    width="1.0823"
                    height="1.1152"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="0.25399999" />
                </filter>

                <linearGradient
                    id="linearGradientValveBody"
                    x1="25.4"
                    x2="27.517"
                    y1="36.142"
                    y2="36.142"
                    gradientTransform="matrix(1 0 0 1 -.00016903 -.00048809)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientValveMaterial"
                />
                <radialGradient
                    id="radialGradientValveBodyTop"
                    cx="26.271"
                    cy="34.022"
                    r="1.0833"
                    gradientTransform="matrix(1.3013 -.031391 .024268 1.006 -8.7402 2.7069)"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#edcd85" offset={0} />
                    <stop stopColor="#b88517" offset={1} />
                </radialGradient>
                <radialGradient
                    id="radialGradientValveTipOpen"
                    cx="26.458"
                    cy="32.808"
                    r=".84667"
                    gradientTransform="matrix(1 0 0 .75 4.8e-7 9.2604)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientValveTipMaterial"
                />
                <radialGradient
                    id="linearGradientValveTipClosed"
                    cx="26.458"
                    cy="32.808"
                    r=".84667"
                    gradientTransform="matrix(1 0 0 .75 2.14e-6 11.165)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientValveTipMaterial"
                />
                <linearGradient
                    id="linearGradientValveTipSpecularOpen"
                    x1="26.458"
                    x2="26.458"
                    y1="33.232"
                    y2="32.808"
                    gradientTransform="translate(4.8e-7 1.0583)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientGoldFade"
                />
                <linearGradient
                    id="linearGradientValveTipSpecularClosed"
                    x1="26.458"
                    x2="26.458"
                    y1="33.232"
                    y2="32.808"
                    gradientTransform="translate(2.14e-6 2.9633)"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientGoldFade"
                />
                <linearGradient
                    id="linearGradientValveBodyStroke"
                    x1="26.458"
                    x2="26.458"
                    y1="38.419"
                    y2="38.1"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientGoldFade"
                />
                <linearGradient
                    id="linearGradientValveBodyTopStroke"
                    x1="26.458"
                    x2="26.458"
                    y1="36.618"
                    y2="35.348"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stopColor="#edcc85" offset={0} />
                    <stop
                        stopColor="#c8921d"
                        stopOpacity=".014052"
                        offset={1}
                    />
                </linearGradient>
                <linearGradient
                    id="linearGradientValveShaft"
                    x1="25.985"
                    x2="26.932"
                    y1="35.004"
                    y2="35.004"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientValveMaterial"
                />
                <linearGradient
                    id="linearGradientValveShaftStroke"
                    x1="26.458"
                    x2="26.458"
                    y1="36.195"
                    y2="35.983"
                    gradientUnits="userSpaceOnUse"
                    xlinkHref="#linearGradientGoldFade"
                />
                <filter
                    id="filterValveBodyTopClosedShadow"
                    x="-.096"
                    y="-.16"
                    width="1.192"
                    height="1.32"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="0.084666663" />
                </filter>
                <filter
                    id="filterValveBodyTopOpenShadow"
                    x="-.18144"
                    y="-.35437"
                    width="1.3629"
                    height="1.7087"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="0.096012333" />
                </filter>
                <filter
                    id="filterValveShadow"
                    x="-.18857"
                    y="-.33"
                    width="1.3771"
                    height="1.66"
                    colorInterpolationFilters="sRGB"
                >
                    <feGaussianBlur stdDeviation="0.23283332" />
                </filter>

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

                <filter id="filterTextShadow" colorInterpolationFilters="sRGB">
                    <feGaussianBlur stdDeviation="0.3" />
                </filter>
            </defs>
        );
    }

    shadow() {
        let count = this.props.labels.length;
        let pipe_shadow_h = (
            <rect
                x="2.1167"
                y="30.692"
                width={31.75 + WaterSwitch.xdist * count}
                height="3.175"
                ry={0}
            />
        );
        let pipe_shadows_v = Array(count);

        for (let i = 0; i < count; i++) {
            pipe_shadows_v[i] = (
                <path
                    transform={"translate(" + WaterSwitch.xdist * i + ")"}
                    key={"vpipe_shadow" + i}
                    d="m26.458 27.517s2.3862 0.2695 3.175 1.0583 1.0583 3.175 1.0583
                    3.175v12.7s0.08814 3.3382-1.0583 4.2333c-0.83418 0.65131-2.1167
                    1.0583-3.1751.0583s-2.1167-1.0583-2.1167-1.0583z"
                />
            );
        }

        return (
            <g transform="translate(0 -13.758)" filter="url(#filterPipeShadow)">
                {pipe_shadow_h}
                {pipe_shadows_v}
            </g>
        );
    }

    pipes() {
        let count = this.props.labels.length;
        let pipe_h = (
            <rect
                x="4.2333"
                y="7.4083"
                width={27.517 + WaterSwitch.xdist * count}
                height="10.583"
                fill="url(#linearGradientPipeH)"
            />
        );
        let flow_window = (
            <g transform="translate(0 -13.758)">
                {/* pipe window */}
                <rect
                    x="8.4667"
                    y="25.4"
                    width="6.35"
                    height="2.1167"
                    fill="#0897f4"
                    stroke="url(#linearGradientWindow)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".212"
                />
                {/* 3 water drops */}
                <path
                    d="m9.525 26.035c1.0586-0.07057 2.1169-0.14113 2.646-0.10599
                0.52903 0.03514 0.52903 0.17628-1e-3 0.21164-0.53004
                0.03535-1.5864-0.03507-2.645-0.10564"
                    fill="#8dbfdc"
                    filter="url(#filterWaterDrop)"
                />
                <path
                    transform="translate(1.6933 .42333)"
                    d="m9.525 26.035c1.0586-0.07057 2.1169-0.14113 2.646-0.10599
                    0.52903 0.03514 0.52903 0.17628-1e-3 0.21164-0.53004
                    0.03535-1.5864-0.03507-2.645-0.10564"
                    fill="#a3cce1"
                    filter="url(#filterWaterDrop)"
                />
                <path
                    transform="translate(-.635 1.0583)"
                    d="m10.16 26.004c0.84695-0.0602 1.6936-0.12037 2.1168-0.08004
                    0.42319 0.04033 0.42319 0.18148-8.16e-4 0.21163-0.42401
                    0.03016-1.2691-0.05063-2.116-0.13159"
                    fill="#8dbfdc"
                    filter="url(#filterWaterDrop)"
                />
            </g>
        );
        let pipe_cap_l = (
            <rect
                x="3.175"
                y="6.35"
                width="1.0583"
                height="12.7"
                fill="url(#linearGradientPipeCapL)"
                stroke="#757575"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth=".1"
            />
        );
        let pipe_cap_r = (
            <rect
                x={31.75 + WaterSwitch.xdist * count}
                y="8.4667"
                width="1.0583"
                height="8.4667"
                fill="url(#linearGradientPipeCapR)"
                stroke="#4a4a4a"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth=".1"
            />
        );

        let pipes_v = Array(count);
        for (let i = 0; i < count; i++) {
            if (this.props.states[i] === true) pipes_v[i] = this.open_pipe(i);
            else pipes_v[i] = this.closed_pipe(i);
        }

        return (
            <g>
                {pipe_h}
                {flow_window}
                {pipe_cap_l}
                {pipe_cap_r}
                {pipes_v}
            </g>
        );
    }

    open_pipe(pos) {
        return (
            <g
                key={"vpipe" + pos}
                transform={"translate(" + WaterSwitch.xdist * pos + " -13.758)"}
            >
                {/* pipe and connection to main */}
                <path
                    d="m23.283 29.633c0.84667-3.3867 5.5033-3.3867 6.35 0v16.933h-6.35z"
                    fill="url(#linearGradientPipeV)"
                    stroke="url(#radialGradientPipeVStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".212"
                />
                {/* filled pipe hole */}
                <ellipse
                    cx="26.458"
                    cy="47.096"
                    rx="3.175"
                    ry="2.54"
                    fill="#4c9ac8"
                    stroke="url(#linearGradientPipeVHoleStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".212"
                />
                <g transform="translate(0 2.6458)">
                    {/* water puddle */}
                    <ellipse
                        transform="matrix(.94591 0 0 .94591 1.4312 2.9769)"
                        cx="26.458"
                        cy="55.033"
                        rx="7.4083"
                        ry="5.2917"
                        fill="#8ec5f3"
                        filter="url(#filterWaterPuddleBlur)"
                        stroke="#cfd5ff"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeOpacity=".637"
                        strokeWidth=".1"
                    />
                    {/* water puddle ripple */}
                    <ellipse
                        transform="matrix(.74404 0 0 .74404 6.7723 14.086)"
                        cx="26.458"
                        cy="55.033"
                        rx="7.4083"
                        ry="5.2917"
                        fill="#89c3f5"
                        filter="url(#filterWaterRippleBlur)"
                        stroke="#dce1ff"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth=".1"
                    />
                    {/* waterfall */}
                    <path
                        d="m23.283 44.45-1.0583 9.525c-0.51804 4.6623 8.9836
                        4.6525 8.4667 0l-1.0583-9.525c-0.69232-3.175-5.6654-3.175-6.35 0z"
                        fill="url(#linearGradientWaterfall)"
                        filter="url(#filterWaterRippleBlur)"
                    />
                </g>
                {/* water specular */}
                <ellipse
                    transform="rotate(3.225)"
                    cx="28.342"
                    cy="49.845"
                    rx="1.5875"
                    ry="7.4083"
                    fill="url(#radialGradientWaterfallSpecular)"
                />
            </g>
        );
    }

    closed_pipe(pos) {
        return (
            <g
                key={"vpipe" + pos}
                transform={"translate(" + WaterSwitch.xdist * pos + " -13.758)"}
            >
                {/* pipe and connection to main */}
                <path
                    d="m23.283 29.633c0.84667-3.3867 5.5033-3.3867 6.35 0v16.933h-6.35z"
                    fill="url(#linearGradientPipeV)"
                    stroke="url(#radialGradientPipeVStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".212"
                />
                {/* empty pipe hole */}
                <ellipse
                    cx="26.458"
                    cy="47.096"
                    rx="3.175"
                    ry="2.54"
                    fill="url(#linearGradientPipeVHole)"
                    stroke="url(#linearGradientPipeVHoleStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".212"
                />
                {/* empty pipe hole shadow */}
                <path
                    d="m26.458 49.636c-1.7535 0-3.175-1.1372-3.175-2.54s1.4215-2.54
                    3.175-2.54 3.175 1.1372 3.175 2.54-1.4215 2.54-3.175 2.54"
                    fill="url(#linearGradientPipeVHoleShadow)"
                />
            </g>
        );
    }

    valves() {
        let count = this.props.labels.length;
        let valves = Array(count);
        for (let i = 0; i < count; i++) {
            valves[i] = this.valve_ctrl(i);
        }
        return valves;
    }

    valve_ctrl(pos) {
        let valve_button = null;
        let valve = null;

        if (this.state.hovered[pos] === true)
            valve_button = this.button_hovered(pos);
        else valve_button = this.button(pos);

        if (this.props.states[pos] === true) valve = this.open_valve(pos);
        else valve = this.closed_valve(pos);

        return (
            <g
                key={"valve" + pos}
                onClick={this.onClick.bind(this, pos)}
                onMouseEnter={this.onMouseEnter.bind(this, pos)}
                onMouseLeave={this.onMouseLeave.bind(this, pos)}
            >
                {valve_button}
                {valve}
            </g>
        );
    }

    open_valve(pos) {
        return (
            <g
                transform={
                    "matrix(1.556 0 0 1.556 " +
                    (-14.711 + WaterSwitch.xdist * pos) +
                    " -34.509)"
                }
            >
                {/* shadow */}
                <ellipse
                    cx="26.458"
                    cy="38.1"
                    rx="1.4817"
                    ry=".84667"
                    fill="#070501"
                    filter="url(#filterValveShadow)"
                    opacity=".5"
                />
                {/* body */}
                <path
                    d="m25.4 35.983h2.1167v2.1167c0 0.42333-2.1167 0.42333-2.1167 0z"
                    fill="url(#linearGradientValveBody)"
                    stroke="url(#linearGradientValveBodyStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
                {/* body top */}
                <ellipse
                    cx="26.458"
                    cy="35.983"
                    rx="1.058"
                    ry=".63502"
                    fill="url(#radialGradientValveBodyTop)"
                    stroke="url(#linearGradientValveBodyTopStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
                {/* shadow top */}
                <ellipse
                    cx="26.458"
                    cy="35.983"
                    rx=".635"
                    ry=".32512"
                    fill="#070501"
                    filter="url(#filterValveBodyTopOpenShadow)"
                    opacity=".5"
                />
                {/* shaft */}
                <path
                    d="m26.035 33.867h0.84667l-1e-6 2.1167c-0.21167
                        0.21167-0.635 0.21167-0.84667 0z"
                    fill="url(#linearGradientValveShaft)"
                    stroke="url(#linearGradientValveShaftStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
                {/* tip */}
                <ellipse
                    cx="26.458"
                    cy="33.867"
                    rx=".84667"
                    ry=".635"
                    fill="url(#radialGradientValveTipOpen)"
                />
                {/* specular */}
                <path
                    d="m27.093 33.867c0 0.2338-0.2843 0.42333-0.635
                        0.42333s-0.635-0.18953-0.635-0.42333"
                    fill="none"
                    stroke="url(#linearGradientValveTipSpecularOpen)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
            </g>
        );
    }

    closed_valve(pos) {
        return (
            <g
                transform={
                    "matrix(1.556 0 0 1.556 " +
                    (-14.711 + WaterSwitch.xdist * pos) +
                    " -34.509)"
                }
            >
                {/* shadow */}
                <ellipse
                    cx="26.458"
                    cy="38.1"
                    rx="1.4817"
                    ry=".84667"
                    fill="#070501"
                    filter="url(#filterValveShadow)"
                    opacity=".5"
                />
                {/* body */}
                <path
                    d="m25.4 35.983h2.1167v2.1167c0 0.42333-2.1167 0.42333-2.1167 0z"
                    fill="url(#linearGradientValveBody)"
                    stroke="url(#linearGradientValveBodyStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
                {/* body top */}
                <ellipse
                    cx="26.458"
                    cy="35.983"
                    rx="1.0583"
                    ry=".63502"
                    fill="url(#radialGradientValveBodyTop)"
                    stroke="url(#linearGradientValveBodyTopStroke)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
                {/* shadow top */}
                <ellipse
                    cx="26.458"
                    cy="35.983"
                    rx=".84667"
                    ry=".51822"
                    fill="#070501"
                    filter="url(#filterValveBodyTopClosedShadow)"
                    opacity=".5"
                />
                {/* tip */}
                <ellipse
                    cx="26.458"
                    cy="35.772"
                    rx=".84667"
                    ry=".635"
                    fill="url(#linearGradientValveTipClosed)"
                />
                */}
                {/* specular */}
                <path
                    d="m27.093 35.772c0 0.2338-0.2843 0.42333-0.635
                        0.42333s-0.635-0.18953-0.635-0.42333"
                    fill="none"
                    stroke="url(#linearGradientValveTipSpecularClosed)"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth=".1"
                />
            </g>
        );
    }

    button(pos) {
        return (
            <g transform={"translate(" + WaterSwitch.xdist * pos + " -13.758)"}>
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
                    x="22.225"
                    y="28.575"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fill="url(#linearGradientButton)"
                />
            </g>
        );
    }

    // TODO: use it correctly
    // TODO: add hover
    button_pushed(pos) {
        return (
            <g transform={"translate(" + WaterSwitch.xdist * pos + " -13.758)"}>
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

    button_hovered(pos) {
        return (
            <g transform={"translate(" + WaterSwitch.xdist * pos + " -13.758)"}>
                <rect
                    x="22.225"
                    y="28.575"
                    width="8.4667"
                    height="14.817"
                    rx="1.4552"
                    fillOpacity=".5082"
                    stroke="#666"
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
                    fill="url(#linearGradientButtonHovered)"
                />
            </g>
        );
    }

    label(pos, text) {
        return (
            <g transform={"translate(" + WaterSwitch.xdist * pos + ")"}>
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

    labels() {
        let count = this.props.labels.length;
        let labels = Array(count);
        for (let i = 0; i < count; i++) {
            labels[i] = this.label(i, this.props.labels[i]);
        }
        return labels;
    }

    render() {
        return (
            // TODO: fit view box to longest label
            <svg viewBox="0 0 135.47, 70">
                {this.defs()}
                {this.shadow()}
                {this.pipes()}
                {this.valves()}
                {this.labels()}
            </svg>
        );
    }

    onMouseEnter = (channel) => {
        let hovered = this.state.hovered;
        hovered[channel] = true;
        this.setState({ hovered: hovered });
    };

    onMouseLeave = (channel) => {
        let hovered = this.state.hovered;
        hovered[channel] = false;
        this.setState({ hovered: hovered });
    };

    onClick = (channel) => {
        this.props.onClick(channel);
    };
}

export default WaterSwitch;
