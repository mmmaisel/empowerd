class WaterApi {
    #token;

    constructor() {
        this.#token = "";
    }

    execute(query, on_success, on_error) {
        let headers = { "Content-Type": "application/json" };
        if (this.#token !== "") {
            headers["Authorization"] = `Bearer ${this.#token}`;
        }

        fetch("graphql", {
            method: "POST",
            headers: headers,
            body: JSON.stringify({
                query: query,
            }),
        })
            .then((response) => response.json())
            .then((response) => {
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

    query(query, on_success, on_error) {
        this.execute(`query{${query}}`, on_success, on_error);
    }

    mutation(mutation, on_success, on_error) {
        this.execute(`mutation{${mutation}}`, on_success, on_error);
    }

    login = (username, password, on_success, on_error) => {
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

    logout = (on_success, on_error) => {
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

    valves = (on_success, on_error) => {
        this.query("valves{id,name,open}", on_success, on_error);
    };
}

export default WaterApi;
