import React, {Component} from 'react';
import WaterSwitch from './WaterSwitch.js'

class Status extends Component
{
    render()
    {
        // TODO: server time, manual trigger, next event
        return (
            <div>
                Status page
                <WaterSwitch count={6} states={[0, 1, 0, 1, 1, 0]}/>
            </div>
        );
    }
}

export default Status;
