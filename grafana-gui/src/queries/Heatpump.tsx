import { Field, Fragment, Query, Timeseries } from "./Query";
import { AggregateProxy, TimeseriesProxy } from "./Proxy";

export class HeatpumpSeries extends Timeseries {
    static basename = "heatpump";
    static time = new Field("time", null);
    static heat = new Field("power_w * cop_pct / 100.0", "heat_w");
    static heat_wh = new Field("heat_wh", null);
    static power = new Field("power_w", null);
    static cop = new Field("cop_pct / 100.0", "cop");
    static d_heat = new Field("MAX(heat_wh)-MIN(heat_wh)", "d_heat_wh");
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
        if (ids.length === 1) {
            return new Field(`heatpump${ids[0]}.heat_w`, `s_heat`);
        } else {
            return new Field(
                ids.map((id) => `COALESCE(heatpump${id}.heat_w, 0)`).join("+"),
                `s_heat`
            );
        }
    }

    static pds_heat(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(
                `MAX(heatpump${ids[0]}.heat_wh)-MIN(heatpump${ids[0]}.heat_wh`,
                "ds_heat_wh"
            );
        } else {
            return new Field(
                ids
                    .map(
                        (id) =>
                            // prettier-ignore
                            `COALESCE(` +
                                `MAX(heatpump${id}.heat_wh)-` +
                                `MIN(heatpump${id}.heat_wh)` +
                            `, 0)`
                    )
                    .join("+"),
                "ds_heat_wh"
            );
        }
    }

    static ps_power(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`heatpump${ids[0]}.power_w`, `s_power`);
        } else {
            return new Field(
                ids.map((id) => `COALESCE(heatpump${id}.power_w, 0)`).join("+"),
                `s_power`
            );
        }
    }

    constructor(id: number) {
        super();
        this.name_ = `heatpump${id}`;
        this.from_ = new Fragment("heatpumps");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(HeatpumpSeries.time);
        return this;
    }

    public heat(alias: string | null): this {
        this.fields_.push(HeatpumpSeries.heat.with_alias(alias));
        return this;
    }

    public heat_wh(alias: string | null): this {
        this.fields_.push(HeatpumpSeries.heat_wh.with_alias(alias));
        return this;
    }

    public d_heat(alias: string | null): this {
        this.fields_.push(HeatpumpSeries.d_heat.with_alias(alias));
        return this;
    }

    public power(alias: string | null): this {
        this.fields_.push(HeatpumpSeries.power.with_alias(alias));
        return this;
    }

    public cop(alias: string | null): this {
        this.fields_.push(HeatpumpSeries.cop.with_alias(alias));
        return this;
    }

    public a_cop(alias: string | null): this {
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
    static query_all(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new HeatpumpSeries(id)
                .time()
                .heat(`"heatpump.heat_w"`)
                .power(`"heatpump.power_w"`)
                .cop(`"heatpump.cop"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id)
                            .time()
                            .heat(null)
                            .power(null)
                            .cop(null)
                            .time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.time,
                    new Field(`"heatpump.heat_w`, null),
                    new Field(`"heatpump.power_w`, null),
                    new Field(`"heatpump.cop`, null),
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
                .heat(`\"heatpump.heat_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id).time().heat(null).time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.time,
                    new Field(`\"heatpump.heat_w\"`, null),
                ])
                .from(
                    new AggregateProxy(HeatpumpSeries, ids, [
                        HeatpumpSeries.ps_heat(ids).with_alias(
                            `\"heatpump.heat_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_dheat_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new HeatpumpSeries(ids[0]).d_heat(null).time_filter();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id)
                            .time()
                            .heat_wh(null)
                            .time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.pds_heat(ids).with_alias(
                        `\"heatpump.heat_wh\"`
                    ),
                ])
                .from(new Fragment(`heatpump${ids[0]}`))
                .joins(
                    HeatpumpSeries.time_join(`heatpump${ids[0]}`, ids.slice(1))
                );
        }
    }

    static query_acop_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new HeatpumpSeries(id)
                .a_cop(`\"heatpump.cop\"`)
                .time_filter();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new HeatpumpSeries(id).time().cop(null).time_filter()
                    )
                )
                .fields([
                    HeatpumpSeries.pa_cop(ids).with_alias(`\"heatpump.cop\"`),
                ])
                .from(new Fragment(`heatpump${ids[0]}`))
                .joins(
                    HeatpumpSeries.time_join(`heatpump${ids[0]}`, ids.slice(1))
                );
        }
    }
}
