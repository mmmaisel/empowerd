export class Fragment {
    public expr: string;

    constructor(expr: string) {
        this.expr = expr;
    }

    public sql(): string {
        return this.expr;
    }
}

export class Field extends Fragment {
    public alias: string;

    constructor(expr: string, alias: string | null) {
        super(expr);
        if (alias === null) {
            this.alias = expr;
        } else {
            this.alias = alias;
        }
    }

    public sql(): string {
        if (this.expr === this.alias) {
            return this.expr;
        } else {
            return `${this.expr} AS ${this.alias}`;
        }
    }

    public with_alias(alias: string | null): Field {
        if (alias === null) {
            return this;
        } else {
            return new Field(this.expr, alias);
        }
    }
}

export class Join extends Fragment {
    public other: string;
    public on: string;

    constructor(typ: string, other: string, on: string) {
        super(typ);
        this.other = other;
        this.on = on;
    }

    public sql(): string {
        return `${this.expr} JOIN ${this.other} ON ${this.on}`;
    }
}

export class Query extends Fragment {
    protected name_: string | null = null;
    protected withs_: Query[] = [];
    protected fields_: Field[] = [];
    protected from_: Fragment = new Fragment("");
    protected joins_: Join[] = [];
    protected wheres_: string[] = [];

    constructor() {
        super("");
    }

    public name(name: string): this {
        this.name_ = name;
        return this;
    }

    public subqueries(withs: Query[]): this {
        this.withs_ = withs;
        return this;
    }

    public fields(fields: Field[]): this {
        this.fields_ = fields;
        return this;
    }

    public from(from: Fragment): this {
        this.from_ = from;
        return this;
    }

    public joins(joins: Join[]): this {
        this.joins_ = joins;
        return this;
    }

    public wheres(wheres: string[]): this {
        this.wheres_ = wheres;
        return this;
    }

    public sql(): string {
        let sql = "";

        if (this.withs_.length !== 0) {
            let withs = this.withs_.map((x) => `${x.name_} AS (${x.sql()})`);
            sql += `WITH ${withs.join(", ")} `;
        }

        sql +=
            `SELECT ${this.fields_.map((f) => f.sql()).join(", ")} ` +
            `FROM ${this.from_.sql()}`;

        if (this.joins_.length !== 0) {
            sql += ` ${this.joins_.map((x) => x.sql()).join(" ")}`;
        }

        if (this.wheres_.length !== 0) {
            sql += ` WHERE ${this.wheres_.join(" ")}`;
        }

        return sql;
    }
}

export class Timeseries extends Query {
    static basename = "";

    public time_filter(): this {
        this.wheres_ = [...this.wheres_, "AND", "$__timeFilter(time)"];
        return this;
    }

    public ordered(): this {
        this.wheres_ = [...this.wheres_, "ORDER BY time"];
        return this;
    }

    public time_not_null(): this {
        this.wheres_ = [...this.wheres_, "time IS NOT NULL"];
        return this;
    }

    static time_join(first: string, ids: number[]): Join[] {
        return ids.map(
            (id) =>
                new Join(
                    "FULL OUTER",
                    `${this.basename}${id}`,
                    `${first}.time = ${this.basename}${id}.time`
                )
        );
    }
}
