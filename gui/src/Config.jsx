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
            edit_row: 0
        };
    }

    onEdit = (id) =>
    {
        this.setState({ edit_mode: true, edit_row: id });
    }

    onSave = (id) =>
    {
        this.setState({ edit_mode: false });
    }

    onInput = (row, col, event) =>
    {
        let data = this.state.data;
        data[row][col] = event.target.value;
        this.setState({ data: data });
    }

    onAdd = () =>
    {
        let data = this.state.data;
        data.push([null, null]);
        this.setState({
            edit_mode: true,
            edit_row: this.state.data.length-1,
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
        // TODO: style buttons
        if(this.state.edit_mode && this.state.edit_row == i)
            return <td>
                <button onClick={this.onSave.bind(this, i)}>Save</button>
                <button disabled="disabled">Delete</button>
            </td>;
        else if(this.state.edit_mode)
            // TODO: style as disabled
            return <td>
                <button disabled="disabled">Edit</button>
                <button disabled="disabled">Delete</button>
            </td>;
        else
            return <td>
                <button onClick={this.onEdit.bind(this, i)}>Edit</button>
                <button onClick={this.onDelete.bind(this, i)}>Delete</button>
            </td>;
    }

    render_cell(row, col)
    {
        // TODO: do not change col width on edit
        if(this.state.edit_mode && this.state.edit_row == row)
            return <td>
                <input type="text" value={this.state.data[row][col]}
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
        // TODO: extract this margin into content or so, it is necessary on any page
        return (
            <div style={{margin: "1vh 2vw 1vh 2vw"}}>
                Config page
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
