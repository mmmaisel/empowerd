export type GraphQlError = {
    message: string;
    locations: Array<{ line: number; column: number }>;
    path: string[];
};

type GraphQlData =
    | string
    | number
    | boolean
    | { [x: string]: GraphQlData }
    | GraphQlData[];

type GraphQlResponse = {
    data: Record<string, GraphQlData>;
    errors: GraphQlError[];
};

export type AvailablePower = {
    id: number;
    name: string;
    threshold: number;
    power: number;
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

export type PoweroffTimer = {
    id: number;
    onTime: number;
    switchId: number;
};

export type Switch = {
    id: number;
    icon: string;
    name: string;
    open: boolean;
};

export class EmpowerdApi {
    #token: string;
    private api_location: string;

    constructor(location: string) {
        this.#token = "";
        this.api_location = `${location.replace(/\/$/, "")}/graphql`;
    }

    private execute(
        query: string,
        on_success: (data: Record<string, GraphQlData>) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        let headers: Record<string, string> = {
            "Content-Type": "application/json",
        };
        if (this.#token !== "") {
            headers["Authorization"] = `Bearer ${this.#token}`;
        }

        fetch(this.api_location, {
            method: "POST",
            headers: headers,
            body: JSON.stringify({
                query: query,
            }),
        })
            .then((response: Response) => response.json())
            .then((response: GraphQlResponse) => {
                if (response.errors) {
                    // TODO: validate received schema
                    // TODO: show error to the user
                    on_error(response.errors);
                } else if (response.data) {
                    // TODO: validate received schema
                    // TODO: add session manager which holds token
                    on_success(response.data);
                } else {
                    on_error(response.errors);
                }
            })
            .catch((error) => {
                // TODO: unify errors
                on_error(error);
            });
    }

    private query(
        query: string,
        on_success: (data: Record<string, GraphQlData>) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        this.execute(`query{${query}}`, on_success, on_error);
    }

    private mutation(
        mutation: string,
        on_success: (data: Record<string, GraphQlData>) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        this.execute(`mutation{${mutation}}`, on_success, on_error);
    }

    public login(
        username: string,
        password: string,
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.mutation(
            `login(username:"${username}",password:"${password}")`,
            (data: GraphQlData) => {
                // TODO:
                this.#token = (data as { login: string }).login;
                on_success();
            },
            (errors: GraphQlError[]) => {
                on_error(errors);
            }
        );
    }

    public logout(
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.mutation(
            "logout",
            (data: GraphQlData) => {
                // TODO: check response
                this.#token = "";
                on_success();
            },
            (errors: GraphQlError[]) => {
                on_error(errors);
            }
        );
    }

    public availablePowers(
        on_success: (power: AvailablePower[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.query(
            "availablePowers{id,name,threshold,power}",
            (data: Record<string, GraphQlData>) => {
                on_success((data as { powers: AvailablePower[] }).powers);
            },
            on_error
        );
    }

    public setAvailablePower(
        id: number,
        threshold: number,
        on_success: (powers: AvailablePower[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.mutation(
            `setAvailablePower(input:{id:${id},threshold:${threshold}}){threshold}`,
            (data: Record<string, GraphQlData>) => {
                on_success((data as { powers: AvailablePower[] }).powers);
            },
            on_error
        );
    }

    public appliances(
        on_success: (appliances: Appliance[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.query(
            "appliances{id,name,forceOnOff}",
            (data: Record<string, GraphQlData>) => {
                on_success((data as { appliances: Appliance[] }).appliances);
            },
            on_error
        );
    }

    public setAppliance(
        id: number,
        force_on_off: string,
        on_success: (appliance: Appliance) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.mutation(
            `setAppliance(input:{id:${id},forceOnOff:${force_on_off}}){forceOnOff}`,
            (data: Record<string, GraphQlData>) => {
                on_success((data as { setAppliance: Appliance }).setAppliance);
            },
            on_error
        );
    }

    public poweroffTimers(
        on_success: (timers: PoweroffTimer[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.query(
            "poweroffTimers{id,onTime,switchId}",
            (data: Record<string, GraphQlData>) => {
                on_success(
                    (data as { poweroffTimers: PoweroffTimer[] }).poweroffTimers
                );
            },
            on_error
        );
    }

    public setPoweroffTimer(
        id: number,
        on_time: number,
        on_success: (timer: PoweroffTimer) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.mutation(
            `setPoweroffTimer(input:{id:${id},onTime:${on_time}}){onTime}`,
            (data: Record<string, GraphQlData>) => {
                on_success(
                    (data as { setPoweroffTimer: PoweroffTimer })
                        .setPoweroffTimer
                );
            },
            on_error
        );
    }

    public switches(
        on_success: (switches: Switch[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        this.query(
            "switches{id,name,icon,open}",
            (data: Record<string, GraphQlData>) => {
                on_success((data as { switches: Switch[] }).switches);
            },
            on_error
        );
    }

    public setSwitch(
        id: number,
        open: boolean,
        on_success: (sw: Switch) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        this.mutation(
            `setSwitch(switch:{id:${id},open:${!!open}}){open}`,
            (data: GraphQlData) => {
                on_success((data as { setSwitch: Switch }).setSwitch);
            },
            on_error
        );
    }
}

/*export class EmpowerdApi {
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

    public poweroffTimers(
        on_success: (timers: PoweroffTimer[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void {
        on_success([
            {
                id: 1,
                onTime: 60,
                switchId: 1,
            },
            {
                id: 2,
                onTime: 60,
                switchId: 3,
            },
        ]);
    }

    setPoweroffTimer = (
        id: number,
        on_time: number,
        on_success: (timer: PoweroffTimer) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        on_success({
            id,
            onTime: on_time,
            switchId: id,
        });
    };

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
}*/
