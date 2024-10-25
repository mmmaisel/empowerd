export type QueryFragment = {
    sources: string[];
    selects: string[];
    table: string;
    columns: string[];
    joins: string[];
};

export class QueryBuilder {
    static timeseries = (fragments: QueryFragment[]): string => {
        let sources = fragments.reduce((acc, frag) => {
            return acc.concat(frag.sources);
        }, Array<string>());

        let selects = fragments.reduce((acc, frag) => {
            return acc.concat(frag.selects);
        }, Array<string>());

        let first_table = fragments[0].table;

        let columns = fragments.reduce((acc, frag) => {
            return acc.concat(frag.columns);
        }, Array<string>());

        let joins = fragments.reduce((acc, frag) => {
            return acc.concat(frag.joins);
        }, Array<string>());

        const sql =
            // prettier-ignore
            `WITH ${sources.join(", ")} ` +
            `SELECT time, ${selects.join(", ")} ` +
            `FROM ( SELECT ` +
                `${first_table}.time AS time, ` +
                `${columns.join(", ")} ` +
                `FROM ${first_table} ` +
                `${joins.join(" ")} ` +
                `OFFSET 0` +
            `) AS x WHERE time IS NOT NULL ORDER BY time`;

        return sql;
    };
}
