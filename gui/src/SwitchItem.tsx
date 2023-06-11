import EmpowerdApi, {
    Appliance,
    GraphQlError,
    Switch,
    TriState,
} from "./EmpowerdApi";

abstract class SwitchItem {
    id: number;
    name: string;
    icon: string;
    state: TriState;
    configHandle: string | null;

    abstract key(): string;
    abstract clone(): SwitchItem;
    abstract toggle(): SwitchItem;
    abstract save(
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
    key(): string {
        return `switch${this.id}`;
    }

    clone(): GpioSwitchItem {
        return new GpioSwitchItem(
            this.id,
            this.name,
            this.icon,
            this.state,
            this.configHandle
        );
    }

    toggle(): SwitchItem {
        let clone = this.clone();
        if (clone.state === TriState.On) clone.state = TriState.Off;
        else clone.state = TriState.On;

        return clone;
    }

    save(
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
                onError(`Setting switch ${this.name} failed.`);
            }
        );
    }
}

export class ApplianceSwitchItem extends SwitchItem {
    key(): string {
        return `appliance${this.id}`;
    }

    clone(): ApplianceSwitchItem {
        return new ApplianceSwitchItem(
            this.id,
            this.name,
            this.icon,
            this.state,
            this.configHandle
        );
    }

    toggle(): SwitchItem {
        let clone = this.clone();
        if (clone.state === TriState.On) clone.state = TriState.Off;
        else if (clone.state === TriState.Off) clone.state = TriState.Auto;
        else clone.state = TriState.On;

        return clone;
    }

    save(
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
                onError(`Setting Appliance ${this.name} failed.`);
            }
        );
    }
}

export class SwitchItemFactory {
    static fromAppliance(a: Appliance): SwitchItem {
        return new ApplianceSwitchItem(
            a.id,
            a.name,
            "Power",
            a.forceOnOff,
            null
        );
    }

    static fromGpioSwitch(sw: Switch): SwitchItem {
        return new GpioSwitchItem(
            sw.id,
            sw.name,
            sw.icon,
            sw.open ? TriState.On : TriState.Off,
            null
        );
    }
}

export default SwitchItem;
