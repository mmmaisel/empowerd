import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

export class BidirMeterSeries extends Timeseries {
    static basename = "bidir_meter";
    static time = new Field("time");
    static power = new Field("power_w");
    static npower = new Field("-power_w", "npower_w");
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
        this.name_ = `${BidirMeterSeries.basename}${id}`;
        this.from_ = new Fragment("bidir_meters");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(BidirMeterSeries.time);
        return this;
    }

    public power(alias: string | null = null): this {
        this.fields_.push(BidirMeterSeries.power.with_alias(alias));
        return this;
    }

    public npower(alias: string | null = null): this {
        this.fields_.push(BidirMeterSeries.npower.with_alias(alias));
        return this;
    }

    public energy_in(alias: string | null = null): this {
        this.fields_.push(BidirMeterSeries.energy_in.with_alias(alias));
        return this;
    }

    public energy_out(alias: string | null = null): this {
        this.fields_.push(BidirMeterSeries.energy_out.with_alias(alias));
        return this;
    }

    public d_energy_in(alias: string | null = null): this {
        this.fields_.push(BidirMeterSeries.d_energy_in.with_alias(alias));
        return this;
    }

    public d_energy_out(alias: string | null = null): this {
        this.fields_.push(BidirMeterSeries.d_energy_out.with_alias(alias));
        return this;
    }
}

export class BidirMeterProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(BidirMeterSeries, ids, fields);
    }
}

export class BidirMeter {
    protected static series = BidirMeterSeries;
    protected static proxy = BidirMeterProxy;

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

    static query_energy_in_out_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new this.series(ids[0])
                .d_energy_in(`\"${this.series.basename}.d_energy_in_wh\"`)
                .d_energy_out(`\"${this.series.basename}.d_energy_out_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id)
                            .time()
                            .energy_in()
                            .energy_out()
                            .time_filter()
                    )
                )
                .fields([
                    this.series.pds_energy_in(
                        ids,
                        `\"${this.series.basename}.d_energy_in_wh\"`
                    ),
                    this.series.pds_energy_out(
                        ids,
                        `\"${this.series.basename}.d_energy_out_wh\"`
                    ),
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
