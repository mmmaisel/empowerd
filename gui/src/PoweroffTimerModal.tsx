import React, { Component, ReactNode } from "react";
import Modal from "react-modal";
import { PoweroffTimer } from "./EmpowerdApi";
import "./Modal.scss";

export type NamedPoweroffTimer = {
    name: String;
    timer: PoweroffTimer;
};

type PoweroffTimerModalProps = {
    timer: NamedPoweroffTimer | null;
    onClose: (on_time: number, canceled: boolean) => void;
};

type PoweroffTimerModalState = {
    on_time: number;
};

class PoweroffTimerModal extends Component<
    PoweroffTimerModalProps,
    PoweroffTimerModalState
> {
    constructor(props: PoweroffTimerModalProps) {
        super(props);

        this.state = {
            on_time: (props.timer && props.timer.timer.onTime) || 0,
        };
    }

    onInputChanged = (event: React.ChangeEvent<HTMLInputElement>): void => {
        let num = Number(event.target.value);
        if (!isNaN(num) && num > 0) this.setState({ on_time: num });
    };

    onCancel = (): void => {
        this.setState({ on_time: 0 });
        this.props.onClose(0, true);
    };

    onApply = (): void => {
        this.setState({ on_time: 0 });
        this.props.onClose(this.state.on_time, false);
    };

    dialog_content(): ReactNode {
        if (this.props.timer === null) return null;

        let named_timer = this.props.timer;
        let on_time = this.state.on_time;
        if (on_time === 0) on_time = this.props.timer.timer.onTime;

        return (
            <React.Fragment>
                <div className="dialogTitle">
                    Configure Poweroff Timer '{named_timer.name}'
                </div>
                <div className="dialogContent">
                    <div className="input-with-label">
                        <div>On Time (seconds):</div>
                        <input
                            type="text"
                            value={on_time}
                            onChange={this.onInputChanged}
                        />
                    </div>
                    <button onClick={this.onCancel}>Cancel</button>
                    <button onClick={this.onApply}>Apply</button>
                </div>
            </React.Fragment>
        );
    }

    render() {
        return (
            <Modal
                isOpen={this.props.timer !== null}
                className="modal-dialog"
                overlayClassName="modal-overlay"
            >
                {this.dialog_content()}
            </Modal>
        );
    }
}

export default PoweroffTimerModal;
