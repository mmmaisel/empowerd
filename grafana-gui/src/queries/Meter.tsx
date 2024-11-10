import { Field, Fragment, Query, Timeseries } from "./Query";
import { AggregateProxy, TimeseriesProxy } from "./Proxy";

export class MeterSeries extends Timeseries {
    static basename = "meter";
    static time = new Field("time", null);
    static power = new Field("power_w", null);
    // TODO: energy_in_wh, energy_out_wh

    static ps_power(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`meter${ids[0]}.power_w`, `s_power_w`);
        } else {
            return new Field(
                ids.map((id) => `COALESCE(meter${id}.power_w, 0)`).join("+"),
                "s_power_w"
            );
        }
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
}
