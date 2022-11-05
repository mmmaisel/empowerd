export type GraphQlError = {
    message: string;
    locations: { line: number; column: number }[];
    path: string[];
};

type GraphQlResponse = {
    data: Record<string, string>;
    errors: GraphQlError[];
};

export type Switch = {
    id: number;
    icon: string;
    name: string;
    open: boolean;
};

class WaterApi {
    #token: string;
    api_location: string;

    constructor(location: string) {
        this.#token = "";
        this.api_location = `${location}graphql`;
    }

    execute(
        query: string,
        on_success: (data: any) => void,
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
        on_success: (data: any) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void {
        this.execute(`query{${query}}`, on_success, on_error);
    }

    mutation(
        mutation: string,
        on_success: (data: any) => void,
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
            (data) => {
                // TODO:
                this.#token = data.login;
                on_success();
            },
            (error) => {
                on_error(error);
            }
        );
    };

    logout = (
        on_success: () => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.mutation(
            "logout",
            (data) => {
                // TODO: check response
                this.#token = "";
                on_success();
            },
            (error) => {
                on_error(error);
            }
        );
    };

    switches = (
        on_success: (switches: Switch[]) => void,
        on_error: (error: GraphQlError[]) => void
    ): void => {
        this.query("switches{id,name,icon,open}", on_success, on_error);
    };

    setSwitch = (
        id: number,
        open: boolean,
        on_success: (switches: Switch[]) => void,
        on_error: (errors: GraphQlError[]) => void
    ): void => {
        this.mutation(
            `setSwitch(switch:{id:${id},open:${!!open}}){open}`,
            on_success,
            on_error
        );
    };
}

export default WaterApi;
