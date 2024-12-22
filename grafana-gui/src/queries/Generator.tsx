import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

export class GeneratorSeries extends Timeseries {
    static basename = "generator";
    static time = new Field("time");
    // power * (1-eta_el)/eta_el * f_Hs_Hi",
    // d_runtime_s * 800 / 3600 * (1-0.138)/0.138 * 1.11
    // === d_runtime_s * 0.222222 * 6.93348
    // === d_runtime_s * 1.540773
    static d_heat_wh = new Field(
        "(MAX(runtime_s)-MIN(runtime_s)) * 1.540773",
        "d_heat_wh"
    );

    static energy = new Field("runtime_s * 0.222222", "energy_wh");

    static d_energy = new Field(
        "(MAX(runtime_s)-MIN(runtime_s)) * 0.222222",
        "d_energy_wh"
    );

    // power * (1-eta_el)/eta_el * f_Hs_Hi
    // power = (1-0.138)/0.138 * 1.11
    // === power * 6.93348
    static heat = new Field("power_w * 6.93348", "heat_w");
    static heat_wh = new Field("energy_wh * 6.93348", "heat_wh");
    // d_runtime_s / 300 * 800 === d_runtime_s * 2.66667
    static power = new Field("power_w");

    static ps_heat(ids: number[]): Field {
        return new SumProxyField(
            this.heat.alias,
            "s_heat_w",
            this.basename,
            ids
        );
    }

    static ps_power(ids: number[]): Field {
        return new SumProxyField(
            this.power.alias,
            "s_power_w",
            this.basename,
            ids
        );
    }

    static pds_heat(ids: number[]): Field {
        return new DeltaSumProxyField(
            this.heat_wh.alias,
            "ds_heat_wh",
            this.basename,
            ids
        );
    }

    static pds_energy(ids: number[]): Field {
        return new DeltaSumProxyField(
            this.energy.alias,
            "ds_energy_wh",
            this.basename,
            ids
        );
    }

    constructor(id: number) {
        super();
        this.name_ = `${GeneratorSeries.basename}${id}`;
        this.from_ = new Fragment("generators");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(GeneratorSeries.time);
        return this;
    }

    public heat(alias: string | null = null): this {
        this.fields_.push(GeneratorSeries.heat.with_alias(alias));
        return this;
    }

    public heat_wh(alias: string | null = null): this {
        this.fields_.push(GeneratorSeries.heat_wh.with_alias(alias));
        return this;
    }

    public d_heat_wh(alias: string | null = null): this {
        this.fields_.push(GeneratorSeries.d_heat_wh.with_alias(alias));
        return this;
    }

    public d_energy(alias: string | null = null): this {
        this.fields_.push(GeneratorSeries.d_energy.with_alias(alias));
        return this;
    }

    public energy(alias: string | null = null): this {
        this.fields_.push(GeneratorSeries.energy.with_alias(alias));
        return this;
    }

    public power(alias: string | null = null): this {
        this.fields_.push(GeneratorSeries.power.with_alias(alias));
        return this;
    }
}

export class GeneratorProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(GeneratorSeries, ids, fields);
    }
}

export class Generator {
    protected static series = GeneratorSeries;
    protected static proxy = GeneratorProxy;

    static query_heat(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .time()
                .heat(`\"${this.series.basename}.heat_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().heat().time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    new Field(`\"${this.series.basename}.heat_w\"`),
                ])
                .from(new this.proxy(ids, [this.series.heat]))
                .time_not_null()
                .ordered();
        }
    }

    static query_heat_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .time()
                .heat(`\"${this.series.basename}.heat_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().heat().time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    new Field(`\"${this.series.basename}.heat_w\"`),
                ])
                .from(
                    new AggregateProxy(this.series, ids, [
                        this.series
                            .ps_heat(ids)
                            .with_alias(`\"${this.series.basename}.heat_w\"`),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_power_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .time()
                .power(`\"${this.series.basename}.power_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().power().time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    new Field(`\"${this.series.basename}.power_w\"`),
                ])
                .from(
                    new AggregateProxy(this.series, ids, [
                        this.series
                            .ps_power(ids)
                            .with_alias(`\"${this.series.basename}.power_w\"`),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_energy_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new this.series(ids[0])
                .d_energy(`\"${this.series.basename}.energy_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().energy().time_filter()
                    )
                )
                .fields([
                    this.series
                        .pds_energy(ids)
                        .with_alias(`\"${this.series.basename}.energy_wh\"`),
                ])
                .from(new Fragment(`${this.series.basename}${ids[0]}`))
                .joins(
                    this.series.time_join(
                        `${this.series.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }

    static query_dheat_wh_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new this.series(ids[0])
                .d_heat_wh(`\"${this.series.basename}.heat_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().heat_wh().time_filter()
                    )
                )
                .fields([
                    this.series
                        .pds_heat(ids)
                        .with_alias(`\"${this.series.basename}.heat_wh\"`),
                ])
                .from(new Fragment(`${this.series.basename}${ids[0]}`))
                .joins(
                    this.series.time_join(
                        `${this.series.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }
}
