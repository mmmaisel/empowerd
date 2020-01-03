import React, {Component} from 'react';
import WaterSwitch from './WaterSwitch.jsx'

class Status extends Component
{
    static channel_count = 6;

    constructor(props)
    {
        super(props);
        this.state =
        {
            valves: Array(Status.channel_count)
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

    render()
    {
        // TODO: server time, manual trigger, next event
        return (
            <div className="mainframe">
                <WaterSwitch count={Status.channel_count}
                    states={this.state.valves}
                    onClick={this.onValve} />
            </div>
        );
    }
}

export default Status;
