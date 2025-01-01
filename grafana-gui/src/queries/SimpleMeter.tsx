import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

export class SimpleMeterSeries extends Timeseries {
    static basename = "simple_meter";
    static time = new Field("time");
    static power = new Field("power_w");
    static energy = new Field("energy_wh");
    static d_energy = new Field("MAX(energy_wh)-MIN(energy_wh)", "d_energy_wh");

    static ps_energy(ids: number[]): Field {
        return new SumProxyField(
            this.energy.expr,
            "s_energy_wh",
            this.basename,
            ids
        );
    }

    static ps_power(ids: number[]): Field {
        return new SumProxyField(
            this.power.expr,
            "s_power_w",
            this.basename,
            ids
        );
    }

    static pds_energy(ids: number[]): Field {
        return new DeltaSumProxyField(
            this.energy.expr,
            "ds_energy_wh",
            this.basename,
            ids
        );
    }

    constructor(id: number) {
        super();
        this.name_ = `${SimpleMeterSeries.basename}${id}`;
        this.from_ = new Fragment("simple_meters");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(SimpleMeterSeries.time);
        return this;
    }

    public power(alias: string | null = null): this {
        this.fields_.push(SimpleMeterSeries.power.with_alias(alias));
        return this;
    }

    public energy(alias: string | null = null): this {
        this.fields_.push(SimpleMeterSeries.energy.with_alias(alias));
        return this;
    }

    public d_energy(alias: string | null = null): this {
        this.fields_.push(SimpleMeterSeries.d_energy.with_alias(alias));
        return this;
    }
}

export class SimpleMeterProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(SimpleMeterSeries, ids, fields);
    }
}

export class SimpleMeter {
    protected static series = SimpleMeterSeries;
    protected static proxy = SimpleMeterProxy;

    // TODO: generalize and extract these methods
    static query_power(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .time()
                .power(`\"${this.series.basename}${id}.power_w\"`)
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
                    ...ids.map(
                        (id) =>
                            new Field(
                                `\"${this.series.basename}${id}.power_w\"`
                            )
                    ),
                ])
                .from(new this.proxy(ids, [this.series.power]))
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

    static query_denergy(id: number): Query {
        return new this.series(id)
            .d_energy(`\"${this.series.basename}${id}.energy_wh\"`)
            .time_filter();
    }

    static query_denergy_sum(ids: number[]): Query {
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
}
