import React, {Component} from 'react';

// TODO: split into classes

import './Config.scss';
import './Widgets.scss';

class Config extends Component
{
    constructor(props)
    {
        super(props);
        this.state =
        {
            data:
            [
                ["foo", "bar"],
                ["bla", "blubb"],
                ["1234", "2345"]
            ],
            edit_mode: false,
            edit_row: 0,
            edit_data: []
        };
    }

    onEdit = (id) =>
    {
        let data = this.state.data[id].map((x)=>{return x;});
        this.setState({ edit_mode: true, edit_row: id, edit_data: data });
    }

    onSave = (id) =>
    {
        let data = this.state.data;
        data[id] = this.state.edit_data;
        this.setState({ edit_mode: false, data: data });
        // TODO: store the data
    }

    onCancel = (id) =>
    {
        // TODO: remove new row on cancel add
        this.setState({ edit_mode: false });
    }

    onInput = (row, col, event) =>
    {
        let data = this.state.edit_data;
        data[col] = event.target.value;
        this.setState({ edit_data: data });
    }

    onAdd = () =>
    {
        let data = this.state.data;
        data.push([null, null]);
        this.setState({
            edit_mode: true,
            edit_row: this.state.data.length-1,
            edit_data: [null, null],
            data: data
        });
    }

    onDelete = (id) =>
    {
        let data = this.state.data;
        data.splice(id, 1);
        this.setState({ data: data });
    }

    render_actions(i)
    {
        if(this.state.edit_mode && this.state.edit_row === i)
            return <td className="actions">
                <button onClick={this.onSave.bind(this, i)}>Save</button>
                <button onClick={this.onCancel.bind(this, i)}>Cancel</button>
            </td>;
        else if(this.state.edit_mode)
            // TODO: style as disabled
            return <td className="actions">
                <button disabled="disabled">Edit</button>
                <button disabled="disabled">Delete</button>
            </td>;
        else
            return <td className="actions">
                <button onClick={this.onEdit.bind(this, i)}>Edit</button>
                <button onClick={this.onDelete.bind(this, i)}>Delete</button>
            </td>;
    }

    render_cell(row, col)
    {
        if(this.state.edit_mode && this.state.edit_row === row)
            return <td className="edit">
                <input type="text" value={this.state.edit_data[col]}
                    onChange={this.onInput.bind(this, row, col)} />
            </td>;
        else
            return <td>{this.state.data[row][col]}</td>;
    }

    render_rows()
    {
        let count = this.state.data.length;
        let rows = Array(count);

        for(let i = 0; i < count; i++)
        {
            rows[i] = (
                <tr key={"row" + i}>
                    {this.render_cell(i, 0)}
                    {this.render_cell(i, 1)}
                    {this.render_actions(i)}
                </tr>
            );
        }
        return rows;
    }

    render()
    {
        return (
            <div className="mainframe">
                <table className="configTable" >
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>Date</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {this.render_rows()}
                    </tbody>
                </table>
                <button disabled={this.state.edit_mode ? "disabled" : ""}
                    onClick={this.onAdd}>Add Row</button>
            </div>
        );
    }
}

export default Config;
