import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

// TODO: derive from bidir_meter
export class BatterySeries extends Timeseries {
    static basename = "battery";
    static time = new Field("time");
    static power = new Field("power_w");
    static npower = new Field("-power_w", "npower_w");
    static charge = new Field("charge_wh");
    static energy_in = new Field("energy_in_wh");
    static energy_out = new Field("energy_out_wh");
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

    static pds_energy_in(ids: number[], alias = "d_energy_in_wh"): Field {
        return new DeltaSumProxyField(
            this.energy_in.alias,
            alias,
            this.basename,
            ids
        );
    }

    static pds_energy_out(ids: number[], alias = "d_energy_out_wh"): Field {
        return new DeltaSumProxyField(
            this.energy_out.alias,
            alias,
            this.basename,
            ids
        );
    }

    constructor(id: number) {
        super();
        this.name_ = `${BatterySeries.basename}${id}`;
        this.from_ = new Fragment("batteries");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(BatterySeries.time);
        return this;
    }

    public power(alias: string | null = null): this {
        this.fields_.push(BatterySeries.power.with_alias(alias));
        return this;
    }

    public npower(alias: string | null = null): this {
        this.fields_.push(BatterySeries.npower.with_alias(alias));
        return this;
    }

    public charge(alias: string | null = null): this {
        this.fields_.push(BatterySeries.charge.with_alias(alias));
        return this;
    }

    public energy_in(alias: string | null = null): this {
        this.fields_.push(BatterySeries.energy_in.with_alias(alias));
        return this;
    }

    public energy_out(alias: string | null = null): this {
        this.fields_.push(BatterySeries.energy_out.with_alias(alias));
        return this;
    }

    public d_energy_in(alias: string | null = null): this {
        this.fields_.push(BatterySeries.d_energy_in.with_alias(alias));
        return this;
    }

    public d_energy_out(alias: string | null = null): this {
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
    protected static series = BatterySeries;

    static query_power(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new BatterySeries(id)
                .time()
                .power(`\"${this.series.basename}${id}.power_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().power().time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    ...ids.map(
                        (id) =>
                            new Field(
                                `\"${this.series.basename}${id}.power_w\"`
                            )
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
                .power(`\"${this.series.basename}.power_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().power().time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    new Field(`\"${this.series.basename}.power_w\"`),
                ])
                .from(
                    new AggregateProxy(BatterySeries, ids, [
                        BatterySeries.ps_power(ids).with_alias(
                            `\"${this.series.basename}.power_w\"`
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
                .charge(`\"${this.series.basename}${id}.charge_wh\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().charge().time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    ...ids.map(
                        (id) =>
                            new Field(
                                `\"${this.series.basename}${id}.charge_wh\"`
                            )
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
                .charge(`\"${this.series.basename}.charge_wh\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id).time().charge().time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    new Field(`\"${this.series.basename}.charge_wh\"`),
                ])
                .from(
                    new AggregateProxy(BatterySeries, ids, [
                        BatterySeries.ps_charge(ids).with_alias(
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
                new BatterySeries(id)
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
                        new BatterySeries(id)
                            .time()
                            .charge()
                            .power()
                            .time_filter()
                    )
                )
                .fields([
                    BatterySeries.time,
                    new Field(`\"${this.series.basename}.charge_wh\"`),
                    new Field(`\"${this.series.basename}.power_w\"`),
                ])
                .from(
                    new AggregateProxy(BatterySeries, ids, [
                        BatterySeries.ps_charge(ids).with_alias(
                            `\"${this.series.basename}.charge_wh\"`
                        ),
                        BatterySeries.ps_power(ids).with_alias(
                            `\"${this.series.basename}.power_w\"`
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
                .d_energy_in(`\"${this.series.basename}.d_energy_in_wh\"`)
                .d_energy_out(`\"${this.series.basename}.d_energy_out_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new BatterySeries(id)
                            .time()
                            .energy_in()
                            .energy_out()
                            .time_filter()
                    )
                )
                .fields([
                    BatterySeries.pds_energy_in(
                        ids,
                        `\"${this.series.basename}.d_energy_in_wh\"`
                    ),
                    BatterySeries.pds_energy_out(
                        ids,
                        `\"${this.series.basename}.d_energy_out_wh\"`
                    ),
                ])
                .from(new Fragment(`${this.series.basename}${ids[0]}`))
                .joins(
                    BatterySeries.time_join(
                        `${this.series.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }
}
