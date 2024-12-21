import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";

export class HeatpumpSeries extends Timeseries {
    static basename = "heatpump";
    static time = new Field("time");
    static energy = new Field("energy_wh");
    static heat = new Field("power_w * cop_pct / 100.0", "heat_w");
    static heat_wh = new Field("heat_wh");
    static power = new Field("power_w");
    static cop = new Field("cop_pct / 100.0", "cop");
    static d_energy = new Field("MAX(energy_wh)-MIN(energy_wh)", "d_energy");
    static d_heat_wh = new Field("MAX(heat_wh)-MIN(heat_wh)", "d_heat_wh");
    static a_cop = new Field("AVG(cop_pct) / 100.0", "cop");

    static pa_cop(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`heatpump${ids[0]}.cop`, `a_cop`);
        } else {
            const cop_sum = ids
                .map((id: number) => `COALESCE(heatpump${id}.cop, 0)`)
                .join("+");
            const cop_cnt = ids
                .map(
                    (id: number) =>
                        `CASE WHEN heatpump${id}.cop > 1 THEN 1 ELSE 0 END`
                )
                .join("+");

            return new Field(`(${cop_sum}) / NULLIF(${cop_cnt}, 0)`, `a_cop`);
        }
    }

    static ps_heat(ids: number[]): Field {
        return new SumProxyField(
            this.heat.alias,
            "s_heat_w",
            this.basename,
            ids
        );
    }

    static pds_heat_wh(ids: number[]): Field {
        return new DeltaSumProxyField(
            this.heat_wh.alias,
            "ds_heat_wh",
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

    constructor(id: number) {
        super();
        this.name_ = `${HeatpumpSeries.basename}${id}`;
        this.from_ = new Fragment("heatpumps");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(HeatpumpSeries.time);
        return this;
    }

    public energy(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.energy.with_alias(alias));
        return this;
    }

    public heat(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.heat.with_alias(alias));
        return this;
    }

    public heat_wh(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.heat_wh.with_alias(alias));
        return this;
    }

    public d_heat_wh(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.d_heat_wh.with_alias(alias));
        return this;
    }

    public power(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.power.with_alias(alias));
        return this;
    }

    public cop(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.cop.with_alias(alias));
        return this;
    }

    public a_cop(alias: string | null = null): this {
        this.fields_.push(HeatpumpSeries.a_cop.with_alias(alias));
        this.wheres_ = [...this.wheres_, "AND", "cop_pct > 100"];
        return this;
    }
}

export class HeatpumpProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(HeatpumpSeries, ids, fields);
    }
}

export class Heatpump {
    protected static series = HeatpumpSeries;

    static query_all(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new HeatpumpSeries(id)
                .time()
                .heat(`"${this.series.basename}.heat_w"`)
                .power(`"${this.series.basename}.power_w"`)
                .cop(`"${this.series.basename}.cop"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id)
                            .time()
                            .heat()
                            .power()
                            .cop()
                            .time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.time,
                    new Field(`"${this.series.basename}.heat_w`),
                    new Field(`"${this.series.basename}.power_w`),
                    new Field(`"${this.series.basename}.cop`),
                ])
                .from(
                    new HeatpumpProxy(ids, [
                        HeatpumpSeries.heat,
                        HeatpumpSeries.power,
                        HeatpumpSeries.cop,
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_heat_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new HeatpumpSeries(id)
                .time()
                .heat(`\"${this.series.basename}.heat_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id).time().heat().time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.time,
                    new Field(`\"${this.series.basename}.heat_w\"`),
                ])
                .from(
                    new AggregateProxy(HeatpumpSeries, ids, [
                        HeatpumpSeries.ps_heat(ids).with_alias(
                            `\"${this.series.basename}.heat_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_dheat_wh_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new HeatpumpSeries(ids[0]).d_heat_wh().time_filter();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id).time().heat_wh().time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.pds_heat_wh(ids).with_alias(
                        `\"${this.series.basename}.heat_wh\"`
                    ),
                ])
                .from(new Fragment(`${this.series.basename}${ids[0]}`))
                .joins(
                    HeatpumpSeries.time_join(
                        `${this.series.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }

    static query_acop_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new HeatpumpSeries(id)
                .a_cop(`\"${this.series.basename}.cop\"`)
                .time_filter();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id).time().cop().time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.pa_cop(ids).with_alias(
                        `\"${this.series.basename}.cop\"`
                    ),
                ])
                .from(new Fragment(`${this.series.basename}${ids[0]}`))
                .joins(
                    HeatpumpSeries.time_join(
                        `${this.series.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }

    static query_power_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new HeatpumpSeries(id)
                .time()
                .power(`\"${this.series.basename}.power_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id).time().power().time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.time,
                    new Field(`\"${this.series.basename}.power_w\"`),
                ])
                .from(
                    new AggregateProxy(HeatpumpSeries, ids, [
                        HeatpumpSeries.ps_power(ids).with_alias(
                            `\"${this.series.basename}.power_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }
}
