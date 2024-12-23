export type GraphQlError = {
    message: string;
    locations: { line: number; column: number }[];
    path: string[];
};

export class EmpowerdApi {
    #token: string;
    private api_location: string;

    constructor(location: string) {
        this.#token = "";
        this.api_location = location;
    }

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
}
