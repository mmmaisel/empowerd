import { Field, Fragment, Query, Timeseries } from "./Query";
import { AggregateProxy, TimeseriesProxy } from "./Proxy";

export class BatterySeries extends Timeseries {
    static basename = "battery";
    static time = new Field("time", null);
    static power = new Field("power_w", null);
    static charge = new Field("charge_wh", null);
    // TODO: energy_in_wh, energy_out_wh

    static ps_power(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`battery${ids[0]}.power_w`, `s_power_w`);
        } else {
            return new Field(
                ids.map((id) => `COALESCE(battery${id}.power_w, 0)`).join("+"),
                "s_power_w"
            );
        }
    }

    static ps_charge(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`battery${ids[0]}.charge_wh`, `s_charge_wh`);
        } else {
            return new Field(
                ids
                    .map((id) => `COALESCE(battery${id}.charge_wh, 0)`)
                    .join("+"),
                "s_charge_wh"
            );
        }
    }

    constructor(id: number) {
        super();
        this.name_ = `battery${id}`;
        this.from_ = new Fragment("batteries");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(BatterySeries.time);
        return this;
    }

    public power(alias: string | null): this {
        this.fields_.push(BatterySeries.power.with_alias(alias));
        return this;
    }

    public charge(alias: string | null): this {
        this.fields_.push(BatterySeries.charge.with_alias(alias));
        return this;
    }
}

export class BatteryProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(BatterySeries, ids, fields);
    }
}

export class Battery {
    static query_power(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new BatterySeries(id)
                .time()
                .power(`"battery${id}.power_w"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    ...ids.map(
                        (id) => new Field(`\"battery${id}.power_w\"`, null)
                    ),
                ])
                .from(new BatteryProxy(ids, [BatterySeries.power]))
                .time_not_null()
                .ordered();
        }
    }

    static query_power_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new BatterySeries(id)
                .time()
                .power(`"battery.power_w"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    new Field(`\"battery.power_w\"`, null),
                ])
                .from(
                    new AggregateProxy(BatterySeries, ids, [
                        BatterySeries.ps_power(ids).with_alias(
                            `\"battery.power_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_charge(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new BatterySeries(id)
                .time()
                .charge(`"battery${id}.charge_wh"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().charge(null).time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    ...ids.map(
                        (id) => new Field(`\"battery${id}.charge_wh\"`, null)
                    ),
                ])
                .from(new BatteryProxy(ids, [BatterySeries.charge]))
                .time_not_null()
                .ordered();
        }
    }

    static query_charge_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new BatterySeries(id)
                .time()
                .charge(`"battery.charge_wh"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().charge(null).time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    new Field(`\"battery.charge_wh\"`, null),
                ])
                .from(
                    new AggregateProxy(BatterySeries, ids, [
                        BatterySeries.ps_charge(ids).with_alias(
                            `\"battery.charge_wh\"`
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
                new BatterySeries(id)
                    .time()
                    // TODO: get rid of aliases
                    .charge(`"battery.charge_wh"`)
                    .power(`"battery.power_w"`)
                    .time_filter()
                    .ordered()
            );
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id)
                            .time()
                            .charge(null)
                            .power(null)
                            .time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    new Field(`\"battery.charge_wh\"`, null),
                    new Field(`\"battery.power_w\"`, null),
                ])
                .from(
                    new AggregateProxy(BatterySeries, ids, [
                        BatterySeries.ps_charge(ids).with_alias(
                            `\"battery.charge_wh\"`
                        ),
                        BatterySeries.ps_power(ids).with_alias(
                            `\"battery.power_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }
}
