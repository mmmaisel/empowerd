import React, {Component} from 'react';
import WaterSwitch from './WaterSwitch.jsx'

// TODO: use React.fragment everywhere where possible

class Status extends Component
{
    static channel_count = 6;
    static labels = [
        "Channel 1", "Channel 2", "Channel 3", "Something",
        "bla bla bla", "anderer channel"
    ];

    constructor(props)
    {
        super(props);
        this.state =
        {
            valves: Array(Status.labels.length)
        };
    }

    onValve = (channel) =>
    {
        // TODO: post state change
        // TODO: then read state from server and update gui
        let valves = this.state.valves;
        if(valves[channel] === true)
            valves[channel] = false;
        else
            valves[channel] = true;

        this.setState({ valves: valves });
    }

    // TODO: show if it is automatically activated
    // TODO: show remaining active time
    // TODO: show channel name

    render()
    {
        // TODO: server time, manual trigger, next event
        return (
            <div className="mainframe">
                <WaterSwitch labels={Status.labels}
                    states={this.state.valves}
                    onClick={this.onValve} />
            </div>
        );
    }
}

export default Status;
