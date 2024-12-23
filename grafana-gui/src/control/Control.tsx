import React, { Component, ReactNode } from "react";
import { SceneObjectBase, SceneObjectState } from "@grafana/scenes";
import "./Control.scss";

type ControlImplProps = {};
type ControlImplState = {};

class ControlImpl extends Component<ControlImplProps, ControlImplState> {
    render(): ReactNode {
        return (
            <>
                <div id="controlFrame">Hello World!</div>
            </>
        );
    }
}

export interface ControlState extends SceneObjectState {}

export class Control extends SceneObjectBase<ControlState> {
    static Component = ControlImpl;

    constructor(state = {}) {
        super(state);
    }
}
