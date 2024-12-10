import { Field, Fragment, Join, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeProxy,
    TimeseriesProxy,
} from "./Proxy";
import { Samples } from "./Samples";

export class SolarSeries extends Timeseries {
    static basename = "solar";
    static time = new Field("time", null);
    static power = new Field("power_w", null);
    static energy = new Field("energy_wh", null);
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
        this.name_ = `solar${id}`;
        this.from_ = new Fragment("simple_meters");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(SolarSeries.time);
        return this;
    }

    public power(alias: string | null): this {
        this.fields_.push(SolarSeries.power.with_alias(alias));
        return this;
    }

    public energy(alias: string | null): this {
        this.fields_.push(SolarSeries.energy.with_alias(alias));
        return this;
    }

    public d_energy(alias: string | null): this {
        this.fields_.push(SolarSeries.d_energy.with_alias(alias));
        return this;
    }
}

export class SolarProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(SolarSeries, ids, fields);
    }
}

export class Solar {
    static query_power(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new SolarSeries(id)
                .time()
                .power(`"solar${id}.power_w"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new SolarSeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    SolarSeries.time,
                    ...ids.map(
                        (id) => new Field(`\"solar${id}.power_w\"`, null)
                    ),
                ])
                .from(new SolarProxy(ids, [SolarSeries.power]))
                .time_not_null()
                .ordered();
        }
    }

    static query_power_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new SolarSeries(id)
                .time()
                .power(`"solar.power_w"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new SolarSeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    SolarSeries.time,
                    new Field(`\"solar.power_w\"`, null),
                ])
                .from(
                    new AggregateProxy(SolarSeries, ids, [
                        SolarSeries.ps_power(ids).with_alias(
                            `\"solar.power_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_energy(id: number): Query {
        return new SolarSeries(id)
            .d_energy(`\"solar${id}.energy_wh\"`)
            .time_filter();
    }

    static query_energy_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new SolarSeries(ids[0])
                .d_energy(`\"solar.energy_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new SolarSeries(id).time().energy(null).time_filter()
                    )
                )
                .fields([
                    SolarSeries.pds_energy(ids).with_alias(
                        `\"solar.energy_wh\"`
                    ),
                ])
                .from(new Fragment(`solar${ids[0]}`))
                .joins(SolarSeries.time_join(`solar${ids[0]}`, ids.slice(1)));
        }
    }

    static query_energy_mon(ids: number[]): Query {
        let solar_query = null;
        if (ids.length === 1) {
            let id = ids[0];
            solar_query = new SolarSeries(id).time().energy(null);
        } else {
            solar_query = new Query()
                .subqueries(
                    ids.map((id) => new SolarSeries(id).time().energy(null))
                )
                .fields([
                    new TimeProxy([`solar${ids[0]}.time`]),
                    SolarSeries.ps_energy(ids).with_alias("energy_wh"),
                ])
                .joins(SolarSeries.time_join(`solar${ids[0]}`, ids.slice(1)))
                .from(new Fragment(`solar${ids[0]}`));
        }

        return (
            new Query()
                .subqueries([
                    new Samples("MONTH", "1 MONTH", "12 HOUR", true),
                    solar_query.name("solar"),
                ])
                .fields([
                    new Field("samples.start", "month"),
                    // TODO: extract this
                    new Field(
                        "solar_next.energy_wh - solar_start.energy_wh",
                        "energy_wh"
                    ),
                ])
                .from(new Fragment("samples"))
                // TODO: extract this
                .joins([
                    new Join(
                        "LEFT OUTER",
                        "solar AS solar_next",
                        "solar_next.time = samples.next"
                    ),
                    new Join(
                        "LEFT OUTER",
                        "solar AS solar_start",
                        "solar_start.time = samples.start"
                    ),
                ])
        );
    }
}
