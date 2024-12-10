import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

export class BatterySeries extends Timeseries {
    static basename = "battery";
    static time = new Field("time", null);
    static power = new Field("power_w", null);
    static npower = new Field("-power_w", "npower_w");
    static charge = new Field("charge_wh", null);
    static energy_in = new Field("energy_in_wh", null);
    static energy_out = new Field("energy_out_wh", null);
    static d_energy_in = new Field(
        "MAX(energy_in_wh)-MIN(energy_in_wh)",
        "d_energy_in_wh"
    );
    static d_energy_out = new Field(
        "MAX(energy_out_wh)-MIN(energy_out_wh)",
        "d_energy_out_wh"
    );

    static ps_power(ids: number[]): Field {
        return new SumProxyField(
            this.power.alias,
            "s_power_w",
            this.basename,
            ids
        );
    }

    static ps_charge(ids: number[]): Field {
        return new SumProxyField(
            this.charge.alias,
            "s_charge_wh",
            this.basename,
            ids
        );
    }

    static pds_energy_in(ids: number[]): Field {
        return new DeltaSumProxyField(
            this.energy_in.alias,
            "d_energy_in_wh",
            this.basename,
            ids
        );
    }

    static pds_energy_out(ids: number[]): Field {
        return new DeltaSumProxyField(
            this.energy_out.alias,
            "d_energy_out_wh",
            this.basename,
            ids
        );
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

    public npower(alias: string | null): this {
        this.fields_.push(BatterySeries.npower.with_alias(alias));
        return this;
    }

    public charge(alias: string | null): this {
        this.fields_.push(BatterySeries.charge.with_alias(alias));
        return this;
    }

    public energy_in(alias: string | null): this {
        this.fields_.push(BatterySeries.energy_in.with_alias(alias));
        return this;
    }

    public energy_out(alias: string | null): this {
        this.fields_.push(BatterySeries.energy_out.with_alias(alias));
        return this;
    }

    public d_energy_in(alias: string | null): this {
        this.fields_.push(BatterySeries.d_energy_in.with_alias(alias));
        return this;
    }

    public d_energy_out(alias: string | null): this {
        this.fields_.push(BatterySeries.d_energy_out.with_alias(alias));
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
                    .npower(`"battery.power_w"`)
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

    static query_energy_in_out_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new BatterySeries(ids[0])
                .d_energy_in(null)
                .d_energy_out(null)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id)
                            .time()
                            .energy_in(null)
                            .energy_out(null)
                            .time_filter()
                    )
                )
                .fields([
                    BatterySeries.pds_energy_in(ids),
                    BatterySeries.pds_energy_out(ids),
                ])
                .from(new Fragment(`${BatterySeries.basename}${ids[0]}`))
                .joins(
                    BatterySeries.time_join(
                        `${BatterySeries.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }
}
