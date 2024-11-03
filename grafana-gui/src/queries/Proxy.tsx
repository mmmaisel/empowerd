import { Field, Fragment, Query } from "./Query";

export class ProxyField extends Field {
    constructor(expr: string) {
        super(expr, null);
    }

    public sql(): string {
        return `${this.expr} AS \"${this.expr}\"`;
    }
}

export class TimeProxy extends Field {
    constructor(expr: string) {
        super(`${expr}.time`, "time");
    }
}

export class ProxyQuery extends Query {
    public sql(): string {
        return `(${super.sql()} OFFSET 0) AS proxy`;
    }
}

export class TimeseriesProxy extends ProxyQuery {
    // series is the class of the Timeseries or its children
    constructor(series: any, ids: number[], fields: Field[]) {
        super();
        this.fields_ = [
            new TimeProxy(`${series.basename}${ids[0]}`),
            ...ids
                .map((id) =>
                    fields.map(
                        (f) =>
                            new ProxyField(`${series.basename}${id}.${f.alias}`)
                    )
                )
                .flat(),
        ];
        this.from_ = new Fragment(`${series.basename}${ids[0]}`);
        this.joins_ = series.time_join(
            `${series.basename}${ids[0]}`,
            ids.slice(1)
        );
    }
}

export class AggregateProxy extends ProxyQuery {
    // series is the class of the Timeseries or its children
    constructor(series: any, ids: number[], fields: Field[]) {
        super();
        this.fields_ = [
            new TimeProxy(`${series.basename}${ids[0]}`),
            ...fields,
        ];
        this.from_ = new Fragment(`${series.basename}${ids[0]}`);
        this.joins_ = series.time_join(
            `${series.basename}${ids[0]}`,
            ids.slice(1)
        );
    }
}
