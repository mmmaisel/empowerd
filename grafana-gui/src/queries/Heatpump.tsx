import { Field, Fragment, Query, Timeseries } from "./Query";
import {
    AggregateProxy,
    DeltaSumProxyField,
    SumProxyField,
    TimeseriesProxy,
} from "./Proxy";
import { SimpleMeter, SimpleMeterSeries } from "./SimpleMeter";

export class HeatpumpSeries extends SimpleMeterSeries {
    static basename = "heatpump";
    static heat = new Field("power_w * cop_pct / 100.0", "heat_w");
    static heat_wh = new Field("heat_wh");
    static cop = new Field("cop_pct / 100.0", "cop");
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

    constructor(id: number) {
        super(id);
        this.name_ = `${HeatpumpSeries.basename}${id}`;
        this.from_ = new Fragment("heatpumps");
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

export class Heatpump extends SimpleMeter {
    protected static series = HeatpumpSeries;
    protected static proxy = HeatpumpProxy;

    static query_all(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
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
                        new this.series(id)
                            .time()
                            .heat()
                            .power()
                            .cop()
                            .time_filter()
                    )
                )
                .fields([
                    this.series.time,
                    new Field(`"${this.series.basename}.heat_w`),
                    new Field(`"${this.series.basename}.power_w`),
                    new Field(`"${this.series.basename}.cop`),
                ])
                .from(
                    new this.proxy(ids, [
                        this.series.heat,
                        this.series.power,
                        this.series.cop,
                    ])
                )
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

    static query_dheat_wh_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new this.series(ids[0]).d_heat_wh().time_filter();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().heat_wh().time_filter()
                    )
                )
                .fields([
                    this.series
                        .pds_heat_wh(ids)
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

    static query_acop_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new this.series(id)
                .a_cop(`\"${this.series.basename}.cop\"`)
                .time_filter();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new this.series(id).time().cop().time_filter()
                    )
                )
                .fields([
                    this.series
                        .pa_cop(ids)
                        .with_alias(`\"${this.series.basename}.cop\"`),
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
