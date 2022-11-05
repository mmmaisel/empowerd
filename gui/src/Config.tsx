import React, { Component, ReactNode } from "react";

// TODO: split into classes
// TODO: cron format, dingle number, comma separated list and range

import "./Config.scss";
import "./Widgets.scss";

import WaterApi from "./WaterApi.jsx";

enum EditMode {
    NORMAL = 0,
    EDIT,
    ADD,
}

type ConfigProps = {
    api: WaterApi;
};

type ConfigState = {
    data: [string, string][];
    mode: EditMode;
    edit_row: number;
    edit_data: any;
};

class Config extends Component<ConfigProps, ConfigState> {
    constructor(props: ConfigProps) {
        super(props);
        this.state = {
            data: [
                ["foo", "bar"],
                ["bla", "blubb"],
                ["1234", "2345"],
            ],
            mode: EditMode.NORMAL,
            edit_row: 0,
            edit_data: [],
        };
    }

    onEdit = (id: number): void => {
        let data: string[] = this.state.data[id].map((x) => {
            return x;
        });
        this.setState({
            mode: EditMode.EDIT,
            edit_row: id,
            edit_data: data,
        });
    };

    onSave = (id: number): void => {
        let data: [string, string][] = this.state.data;
        if (this.state.mode === EditMode.ADD) data.push(this.state.edit_data);
        else data[id] = this.state.edit_data;
        this.setState({ mode: EditMode.NORMAL, data: data });
        // TODO: store the data
    };

    onCancel = (id: number): void => {
        // TODO: remove new row on cancel add
        this.setState({ mode: EditMode.NORMAL });
    };

    onInput = (
        col: number,
        event: React.ChangeEvent<HTMLInputElement>
    ): void => {
        let data: string[] = this.state.edit_data;
        data[col] = event.target.value;
        this.setState({ edit_data: data });
    };

    onAdd = (): void => {
        let data: [string, string][] = this.state.data;
        this.setState({
            mode: EditMode.ADD,
            edit_row: this.state.data.length,
            edit_data: [null, null],
            data: data,
        });
    };

    onDelete = (id: number): void => {
        let data: [string, string][] = this.state.data;
        data.splice(id, 1);
        this.setState({ data: data });
    };

    render_actions(i: number): ReactNode {
        if (this.state.mode !== EditMode.NORMAL && this.state.edit_row === i)
            return (
                <td className="actions">
                    <button onClick={this.onSave.bind(this, i)}>Save</button>
                    <button onClick={this.onCancel.bind(this, i)}>
                        Cancel
                    </button>
                </td>
            );
        else if (this.state.mode !== EditMode.NORMAL)
            // TODO: style as disabled
            return (
                <td className="actions">
                    <button disabled={true}>Edit</button>
                    <button disabled={true}>Delete</button>
                </td>
            );
        else
            return (
                <td className="actions">
                    <button onClick={this.onEdit.bind(this, i)}>Edit</button>
                    <button onClick={this.onDelete.bind(this, i)}>
                        Delete
                    </button>
                </td>
            );
    }

    render_cell(row: number, col: number, data: string): ReactNode {
        if (this.state.mode !== EditMode.NORMAL && this.state.edit_row === row)
            return (
                <td className="edit">
                    <input
                        type="text"
                        value={this.state.edit_data[col]}
                        onChange={this.onInput.bind(this, col)}
                    />
                </td>
            );
        else return <td>{data}</td>;
    }

    render_rows(): ReactNode {
        let count: number = this.state.data.length;
        let rows: ReactNode[] = Array<ReactNode>(count);

        for (let i = 0; i < count; i++) {
            rows[i] = (
                <tr key={"row" + i}>
                    {this.render_cell(i, 0, this.state.data[i][0])}
                    {this.render_cell(i, 1, this.state.data[i][1])}
                    {this.render_actions(i)}
                </tr>
            );
        }
        if (this.state.mode === EditMode.ADD) {
            rows.push(
                <tr key="rowadd">
                    {this.render_cell(count, 0, this.state.edit_data[0])}
                    {this.render_cell(count, 1, this.state.edit_data[1])}
                    {this.render_actions(count)}
                </tr>
            );
        }
        return rows;
    }

    render(): ReactNode {
        return (
            <div className="mainframe">
                <table className="configTable">
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>Date</th>
                            <th style={{ width: "20vw" }}>Actions</th>
                        </tr>
                    </thead>
                    <tbody>{this.render_rows()}</tbody>
                </table>
                <button
                    disabled={this.state.mode !== EditMode.NORMAL}
                    onClick={this.onAdd}
                >
                    Add Row
                </button>
            </div>
        );
    }
}

export default Config;
