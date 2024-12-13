import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

// TODO: BidirMeter class
export class MeterSeries extends Timeseries {
    static basename = "meter";
    static time = new Field("time", null);
    static power = new Field("power_w", null);
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

    static pds_energy_in(ids: number[], alias = "d_energy_wh_in"): Field {
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
        this.name_ = `meter${id}`;
        this.from_ = new Fragment("bidir_meters");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(MeterSeries.time);
        return this;
    }

    public power(alias: string | null): this {
        this.fields_.push(MeterSeries.power.with_alias(alias));
        return this;
    }

    public energy_in(alias: string | null): this {
        this.fields_.push(MeterSeries.energy_in.with_alias(alias));
        return this;
    }

    public energy_out(alias: string | null): this {
        this.fields_.push(MeterSeries.energy_out.with_alias(alias));
        return this;
    }

    public d_energy_in(alias: string | null): this {
        this.fields_.push(MeterSeries.d_energy_in.with_alias(alias));
        return this;
    }

    public d_energy_out(alias: string | null): this {
        this.fields_.push(MeterSeries.d_energy_out.with_alias(alias));
        return this;
    }
}

export class MeterProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(MeterSeries, ids, fields);
    }
}

export class Meter {
    static query_power(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new MeterSeries(id)
                .time()
                .power(`"meter${id}.power_w"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new MeterSeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    MeterSeries.time,
                    ...ids.map(
                        (id) => new Field(`\"meter${id}.power_w\"`, null)
                    ),
                ])
                .from(new MeterProxy(ids, [MeterSeries.power]))
                .time_not_null()
                .ordered();
        }
    }

    static query_power_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new MeterSeries(id)
                .time()
                .power(`"meter.power_w"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new MeterSeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    MeterSeries.time,
                    new Field(`\"meter.power_w\"`, null),
                ])
                .from(
                    new AggregateProxy(MeterSeries, ids, [
                        MeterSeries.ps_power(ids).with_alias(
                            `\"meter.power_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_energy_in_out_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new MeterSeries(ids[0])
                .d_energy_in(`\"meter.d_energy_in_wh\"`)
                .d_energy_out(`\"meter.d_energy_out_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new MeterSeries(id)
                            .time()
                            .energy_in(null)
                            .energy_out(null)
                            .time_filter()
                    )
                )
                .fields([
                    MeterSeries.pds_energy_in(ids, `\"meter.d_energy_in_wh\"`),
                    MeterSeries.pds_energy_out(
                        ids,
                        `\"meter.d_energy_out_wh\"`
                    ),
                ])
                .from(new Fragment(`${MeterSeries.basename}${ids[0]}`))
                .joins(
                    MeterSeries.time_join(
                        `${MeterSeries.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }
}
