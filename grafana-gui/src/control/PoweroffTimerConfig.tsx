import React, { Component, ReactNode } from "react";
import Modal from "react-modal";
import { PoweroffTimer } from "./EmpowerdApi";
import { t } from "../i18n";
import "./Modal.scss";

export type NamedPoweroffTimer = {
    name: String;
    timer: PoweroffTimer;
};

type PoweroffTimerConfigProps = {
    timer: NamedPoweroffTimer | null;
    onClose: (on_time: number | null, canceled: boolean) => void;
};

type PoweroffTimerConfigState = {
    on_time: number | null;
};

export class PoweroffTimerConfig extends Component<
    PoweroffTimerConfigProps,
    PoweroffTimerConfigState
> {
    constructor(props: PoweroffTimerConfigProps) {
        super(props);

        this.state = {
            on_time: (props.timer && props.timer.timer.onTime) || null,
        };
    }

    public onInputChanged(event: React.ChangeEvent<HTMLInputElement>): void {
        let num = Number(event.target.value);
        if (!isNaN(num) && num > 0) {
            this.setState({ on_time: num });
        }
    }

    public onCancel(): void {
        this.setState({ on_time: null });
        this.props.onClose(null, true);
    }

    public onApply(): void {
        this.props.onClose(this.state.on_time, false);
        this.setState({ on_time: null });
    }

    private dialog_content(): ReactNode {
        if (this.props.timer === null) {
            return null;
        }

        let named_timer = this.props.timer;
        let on_time = this.state.on_time || this.props.timer.timer.onTime;

        return (
            <React.Fragment>
                <div className="dialogTitle">
                    {t("config-pwroff-timer", { name: named_timer.name })}
                </div>
                <div className="dialogContent">
                    <div className="modalInputWithLabel">
                        <div>{t("pwroff-on-time")}:</div>
                        <input
                            className="dialogInput"
                            type="text"
                            value={on_time}
                            onChange={this.onInputChanged.bind(this)}
                        />
                    </div>
                    <button
                        className="dialogButton"
                        onClick={this.onCancel.bind(this)}
                    >
                        {t("cancel")}
                    </button>
                    <button
                        className="dialogButton"
                        onClick={this.onApply.bind(this)}
                    >
                        {t("apply")}
                    </button>
                </div>
            </React.Fragment>
        );
    }

    public render(): ReactNode {
        return (
            <Modal
                appElement={document.body}
                isOpen={this.props.timer !== null}
                className="modal-dialog"
                overlayClassName="modal-overlay"
            >
                {this.dialog_content()}
            </Modal>
        );
    }
}
