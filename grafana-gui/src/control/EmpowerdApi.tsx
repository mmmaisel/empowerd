export type GraphQlError = {
    message: string;
    locations: { line: number; column: number }[];
    path: string[];
};

export enum TriState {
    On = "ON",
    Off = "OFF",
    Auto = "AUTO",
}

export type Appliance = {
    id: number;
    name: string;
    forceOnOff: TriState;
};

export type Switch = {
    id: number;
    icon: string;
    name: string;
    open: boolean;
};

export class EmpowerdApi {
    constructor(location: string) {}

    public login(
        username: string,
        password: string,
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        on_success();
    }

    public logout(
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        on_success();
    }

    public appliances(
        on_success: (appliances: Appliance[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        on_success([
            {
                id: 1,
                name: "app1",
                forceOnOff: TriState.On,
            },
            {
                id: 2,
                name: "app2",
                forceOnOff: TriState.Off,
            },
            {
                id: 3,
                name: "app3",
                forceOnOff: TriState.Auto,
            },
        ]);
    }

    public setAppliance(
        id: number,
        force_on_off: string,
        on_success: (appliance: Appliance) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        on_success({ id, forceOnOff: force_on_off } as Appliance);
    }

    public switches(
        on_success: (switches: Switch[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        on_success([
            {
                id: 1,
                name: "gpio1",
                icon: "Valve",
                open: false,
            },
            {
                id: 2,
                name: "gpio2",
                icon: "Valve",
                open: true,
            },
            {
                id: 3,
                name: "gpio3",
                icon: "Power",
                open: false,
            },
            {
                id: 4,
                name: "gpio4",
                icon: "Power",
                open: true,
            },
        ]);
    }

    public setSwitch(
        id: number,
        open: boolean,
        on_success: (sw: Switch) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        on_success({ id, open } as Switch);
    }
}
