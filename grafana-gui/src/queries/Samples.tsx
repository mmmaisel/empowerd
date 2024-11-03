import { Query } from "./Query";

export class Samples extends Query {
    constructor() {
        super();
        this.name_ = "samples";
    }

    public sql(): string {
        const sql =
            // prettier-ignore
            "WITH months AS (" +
                "SELECT GENERATE_SERIES(" +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeFrom())," +
                    "DATE_TRUNC('MONTH', TIMESTAMP $__timeTo())," +
                    "INTERVAL '1 MONTH'" +
                ") AS month" +
            ") " +
            "SELECT " +
            "month + INTERVAL '12 HOUR' AS start," +
            "month + INTERVAL '1 MONTH' + INTERVAL '12 HOUR' AS next " +
            "FROM months";

        return sql;
    }
}
