import React, {Component} from 'react';

// TODO: split into classes
// TODO: cron format, dingle number, comma separated list and range

import './Config.scss';
import './Widgets.scss';

class Config extends Component
{
    static MODE_NORMAL = 0;
    static MODE_EDIT = 1;
    static MODE_ADD = 2;

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
            mode: Config.MODE_NORMAL,
            edit_row: 0,
            edit_data: []
        };
    }

    onEdit = (id) =>
    {
        let data = this.state.data[id].map((x)=>{return x;});
        this.setState({ mode: Config.MODE_EDIT,
            edit_row: id, edit_data: data });
    }

    onSave = (id) =>
    {
        let data = this.state.data;
        if(this.state.mode === Config.MODE_ADD)
            data.push(this.state.edit_data);
        else
            data[id] = this.state.edit_data;
        this.setState({ mode: Config.MODE_NORMAL, data: data });
        // TODO: store the data
    }

    onCancel = (id) =>
    {
        // TODO: remove new row on cancel add
        this.setState({ mode: Config.MODE_NORMAL });
    }

    onInput = (col, event) =>
    {
        let data = this.state.edit_data;
        data[col] = event.target.value;
        this.setState({ edit_data: data });
    }

    onAdd = () =>
    {
        let data = this.state.data;
        this.setState({
            mode: Config.MODE_ADD,
            edit_row: this.state.data.length,
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
        if(this.state.mode !== Config.MODE_NORMAL &&
                this.state.edit_row === i)
            return <td className="actions">
                <button onClick={this.onSave.bind(this, i)}>Save</button>
                <button onClick={this.onCancel.bind(this, i)}>Cancel</button>
            </td>;
        else if(this.state.mode !== Config.MODE_NORMAL)
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

    render_cell(row, col, data)
    {
        if(this.state.mode !== Config.MODE_NORMAL &&
                this.state.edit_row === row)
            return <td className="edit">
                <input type="text" value={this.state.edit_data[col]}
                    onChange={this.onInput.bind(this, col)} />
            </td>;
        else
            return <td>{data}</td>;
    }

    render_rows()
    {
        let count = this.state.data.length;
        let rows = Array(count);

        for(let i = 0; i < count; i++)
        {
            rows[i] = (
                <tr key={"row" + i}>
                    {this.render_cell(i, 0, this.state.data[i][0])}
                    {this.render_cell(i, 1, this.state.data[i][1])}
                    {this.render_actions(i)}
                </tr>
            );
        }
        if(this.state.mode === Config.MODE_ADD)
        {
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
                <button disabled={this.state.mode !== Config.MODE_NORMAL
                        ? "disabled" : ""}
                    onClick={this.onAdd}>Add Row</button>
            </div>
        );
    }
}

export default Config;
