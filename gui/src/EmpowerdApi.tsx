export type GraphQlError = {
    message: string;
    locations: { line: number; column: number }[];
    path: string[];
};

type GraphQlData =
    | string
    | number
    | boolean
    | { [x: string]: GraphQlData }
    | Array<GraphQlData>;

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

class EmpowerdApi {
    #token: string;
    api_location: string;

    constructor(location: string) {
        this.#token = "";
        this.api_location = `${location}graphql`;
    }

    execute(
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

    query(
        query: string,
        on_success: (data: Record<string, GraphQlData>) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        this.execute(`query{${query}}`, on_success, on_error);
    }

    mutation(
        mutation: string,
        on_success: (data: Record<string, GraphQlData>) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        this.execute(`mutation{${mutation}}`, on_success, on_error);
    }

    login = (
        username: string,
        password: string,
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
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
    };

    logout = (
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
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
    };

    availablePowers = (
        on_success: (power: AvailablePower[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.query(
            "availablePowers{id,name,threshold,power}",
            (data: Record<string, GraphQlData>) => {
                on_success((data as { powers: AvailablePower[] }).powers);
            },
            on_error
        );
    };

    setAvailablePower = (
        id: number,
        threshold: number,
        on_success: (powers: AvailablePower[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.mutation(
            `setAvailablePower(input:{id:${id},threshold:${threshold}}){threshold}`,
            (data: Record<string, GraphQlData>) => {
                on_success((data as { powers: AvailablePower[] }).powers);
            },
            on_error
        );
    };

    appliances = (
        on_success: (appliances: Appliance[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.query(
            "appliances{id,name,forceOnOff}",
            (data: Record<string, GraphQlData>) => {
                on_success((data as { appliances: Appliance[] }).appliances);
            },
            on_error
        );
    };

    setAppliance = (
        id: number,
        force_on_off: string,
        on_success: (appliance: Appliance) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.mutation(
            `setAppliance(input:{id:${id},forceOnOff:${force_on_off}}){forceOnOff}`,
            (data: Record<string, GraphQlData>) => {
                on_success((data as { setAppliance: Appliance }).setAppliance);
            },
            on_error
        );
    };

    poweroffTimers = (
        on_success: (timers: PoweroffTimer[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.query(
            "poweroffTimers{id,onTime,switchId}",
            (data: Record<string, GraphQlData>) => {
                on_success(
                    (data as { poweroffTimers: PoweroffTimer[] }).poweroffTimers
                );
            },
            on_error
        );
    };

    setPoweroffTimer = (
        id: number,
        on_time: number,
        on_success: (timer: PoweroffTimer) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
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
    };

    switches = (
        on_success: (switches: Switch[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.query(
            "switches{id,name,icon,open}",
            (data: Record<string, GraphQlData>) => {
                on_success((data as { switches: Switch[] }).switches);
            },
            on_error
        );
    };

    setSwitch = (
        id: number,
        open: boolean,
        on_success: (sw: Switch) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void => {
        this.mutation(
            `setSwitch(switch:{id:${id},open:${!!open}}){open}`,
            (data: GraphQlData) => {
                on_success((data as { setSwitch: Switch }).setSwitch);
            },
            on_error
        );
    };
}

export default EmpowerdApi;
