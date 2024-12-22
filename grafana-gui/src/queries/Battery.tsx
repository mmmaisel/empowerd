import { Fragment, Field, Query, Timeseries } from "./Query";
import { AggregateProxy, SumProxyField, TimeseriesProxy } from "./Proxy";
import { BidirMeter, BidirMeterSeries } from "./BidirMeter";

export class BatterySeries extends BidirMeterSeries {
    static basename = "battery";
    static charge = new Field("charge_wh");

    static ps_charge(ids: number[]): Field {
        return new SumProxyField(
            this.charge.alias,
            "s_charge_wh",
            this.basename,
            ids
        );
    }

    constructor(id: number) {
        super(id);
        this.from_ = new Fragment("batteries");
        this.name_ = `${BatterySeries.basename}${id}`;
    }

    public charge(alias: string | null = null): this {
        this.fields_.push(BatterySeries.charge.with_alias(alias));
        return this;
    }
}

export class BatteryProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(BatterySeries, ids, fields);
    }
}

export class Battery extends BidirMeter {
    protected static series = BatterySeries;
    protected static proxy = BatteryProxy;

    static query_charge(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .time()
                .charge(`\"${this.series.basename}${id}.charge_wh\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().charge().time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    ...ids.map(
                        (id) =>
                            new Field(
                                `\"${this.series.basename}${id}.charge_wh\"`
                            )
                    ),
                ])
                .from(new this.proxy(ids, [this.series.charge]))
                .time_not_null()
                .ordered();
        }
    }

    static query_charge_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .time()
                .charge(`\"${this.series.basename}.charge_wh\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().charge().time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    new Field(`\"${this.series.basename}.charge_wh\"`),
                ])
                .from(
                    new AggregateProxy(this.series, ids, [
                        this.series
                            .ps_charge(ids)
                            .with_alias(
                                `\"${this.series.basename}.charge_wh\"`
                            ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_power_charge_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return (
                new this.series(id)
                    .time()
                    // TODO: get rid of aliases
                    .charge(`\"${this.series.basename}.charge_wh\"`)
                    .npower(`\"${this.series.basename}.power_w\"`)
                    .time_filter()
                    .ordered()
            );
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id)
                            .time()
                            .charge()
                            .power()
                            .time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    new Field(`\"${this.series.basename}.charge_wh\"`),
                    new Field(`\"${this.series.basename}.power_w\"`),
                ])
                .from(
                    new AggregateProxy(this.series, ids, [
                        this.series
                            .ps_charge(ids)
                            .with_alias(
                                `\"${this.series.basename}.charge_wh\"`
                            ),
                        this.series
                            .ps_power(ids)
                            .with_alias(`\"${this.series.basename}.power_w\"`),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }
}
