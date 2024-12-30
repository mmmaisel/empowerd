import {
    EmpowerdApi,
    Appliance,
    GraphQlError,
    Switch,
    TriState,
} from "./EmpowerdApi";
import { t } from "../i18n";

export abstract class SwitchItem {
    protected id: number;
    public name: string;
    public icon: string;
    public state: TriState;
    public configHandle: string | null;

    public abstract key(): string;
    public abstract clone(): SwitchItem;
    public abstract toggle(): SwitchItem;
    public abstract save(
        api: EmpowerdApi,
        onSuccess: (x: SwitchItem) => void,
        onError: (e: string) => void
    ): void;

    constructor(
        id: number,
        name: string,
        icon: string,
        state: TriState,
        configHandle: string | null
    ) {
        this.id = id;
        this.name = name;
        this.icon = icon;
        this.state = state;
        this.configHandle = configHandle;
    }
}

export class GpioSwitchItem extends SwitchItem {
    public key(): string {
        return `switch${this.id}`;
    }

    public clone(): GpioSwitchItem {
        return new GpioSwitchItem(
            this.id,
            this.name,
            this.icon,
            this.state,
            this.configHandle
        );
    }

    public toggle(): SwitchItem {
        let clone = this.clone();
        if (clone.state === TriState.On) clone.state = TriState.Off;
        else clone.state = TriState.On;

        return clone;
    }

    public save(
        api: EmpowerdApi,
        onSuccess: (x: SwitchItem) => void,
        onError: (e: string) => void
    ): void {
        api.setSwitch(
            this.id,
            this.state === TriState.On,
            (response: Switch) => {
                let clone = this.clone();
                clone.state = response.open ? TriState.On : TriState.Off;
                onSuccess(clone);
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
                onError(t("switch-failed", { name: this.name }));
            }
        );
    }
}

export class ApplianceSwitchItem extends SwitchItem {
    public key(): string {
        return `appliance${this.id}`;
    }

    public clone(): ApplianceSwitchItem {
        return new ApplianceSwitchItem(
            this.id,
            this.name,
            this.icon,
            this.state,
            this.configHandle
        );
    }

    public toggle(): SwitchItem {
        let clone = this.clone();
        if (clone.state === TriState.On) clone.state = TriState.Off;
        else if (clone.state === TriState.Off) clone.state = TriState.Auto;
        else clone.state = TriState.On;

        return clone;
    }

    public save(
        api: EmpowerdApi,
        onSuccess: (x: SwitchItem) => void,
        onError: (e: string) => void
    ): void {
        api.setAppliance(
            this.id,
            this.state,
            (response: Appliance) => {
                let clone = this.clone();
                clone.state = response.forceOnOff;
                onSuccess(clone);
            },
            (errors: GraphQlError[]) => {
                console.log(errors);
                onError(t("appliance-failed", { name: this.name }));
            }
        );
    }
}

export class SwitchItemFactory {
    public static fromAppliance(a: Appliance): SwitchItem {
        return new ApplianceSwitchItem(
            a.id,
            a.name,
            "Power",
            a.forceOnOff,
            null
        );
    }

    public static fromGpioSwitch(sw: Switch): SwitchItem {
        return new GpioSwitchItem(
            sw.id,
            sw.name,
            sw.icon,
            sw.open ? TriState.On : TriState.Off,
            null
        );
    }
}
