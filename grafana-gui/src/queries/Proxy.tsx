import { Field, Fragment, Query, Timeseries } from "./Query";

export class ProxyField extends Field {
    constructor(expr: string) {
        super(expr, null);
    }

    public sql(): string {
        return `${this.expr} AS \"${this.expr}\"`;
    }
}

export class SumProxyField extends Field {
    constructor(expr: string, alias: string, basename: string, ids: number[]) {
        if (ids.length === 1) {
            super(`${basename}${ids[0]}.${expr}`, alias);
        } else {
            super(
                ids
                    .map((id) => `COALESCE(${basename}${id}.${expr}, 0)`)
                    .join("+"),
                alias
            );
        }
    }
}

export class DeltaSumProxyField extends Field {
    constructor(expr: string, alias: string, basename: string, ids: number[]) {
        if (ids.length === 1) {
            super(
                `MAX(${basename}${ids[0]}.${expr})-` +
                    `MIN(${basename}${ids[0]}.${expr})`,
                alias
            );
        } else {
            super(
                ids
                    .map(
                        (id) =>
                            // prettier-ignore
                            `COALESCE(` +
                                `MAX(${basename}${id}.${expr})-` +
                                `MIN(${basename}${id}.${expr}), ` +
                            `0)`
                    )
                    .join("+"),
                alias
            );
        }
    }
}

export class TimeProxy extends Field {
    constructor(times: string[]) {
        if (times.length === 1) {
            super(times[0], "time");
        } else {
            super(`COALESCE(${times.join(", ")})`, "time");
        }
    }

    public static from_series(series: Timeseries[]): TimeProxy {
        let times = series.map((x) => `${x.get_name()}.time`);
        return new TimeProxy(times);
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
            new TimeProxy(ids.map((id) => `${series.basename}${id}.time`)),
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
            new TimeProxy(ids.map((id) => `${series.basename}${id}.time`)),
            ...fields,
        ];
        this.from_ = new Fragment(`${series.basename}${ids[0]}`);
        this.joins_ = series.time_join(
            `${series.basename}${ids[0]}`,
            ids.slice(1)
        );
    }
}
