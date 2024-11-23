import { Field, Fragment, Query, Timeseries } from "./Query";
import { AggregateProxy, TimeseriesProxy } from "./Proxy";

export class GeneratorSeries extends Timeseries {
    static basename = "generator";
    static time = new Field("time", null);
    // power * (1-eta_el)/eta_el * f_Hs_Hi",
    // d_runtime_s / 300 * 800 * (1-0.138)/0.138 * 1.11
    // === d_runtime_s * 2.66667 * 6.93348
    // === d_runtime_s * 18.48928
    static d_heat = new Field(
        "(MAX(runtime_s)-MIN(runtime_s)) * 18.48928",
        "heat_wh"
    );

    static d_energy = new Field(
        "(MAX(runtime_s)-MIN(runtime_s)) * 0.222222",
        "energy_wh"
    );

    // power * (1-eta_el)/eta_el * f_Hs_Hi
    // power = (1-0.138)/0.138 * 1.11
    // === power * 6.93348
    static heat = new Field("power_w * 6.93348", "heat_w");
    static heat_wh = new Field("energy_wh * 6.93348", "heat_wh");
    // d_runtime_s / 300 * 800 === d_runtime_s * 2.66667
    static power = new Field("power_w", null);

    static ps_heat(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`generator${ids[0]}.heat_w`, `s_heat`);
        } else {
            return new Field(
                ids.map((id) => `COALESCE(generator${id}.heat_w, 0)`).join("+"),
                `s_heat`
            );
        }
    }

    static ps_power(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`generator${ids[0]}.power_w`, `s_power`);
        } else {
            return new Field(
                ids
                    .map((id) => `COALESCE(generator${id}.power_w, 0)`)
                    .join("+"),
                `s_power`
            );
        }
    }

    static pds_heat(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`generator${ids[0]}.heat_wh`, `s_heat`);
        } else {
            return new Field(
                ids
                    .map(
                        (id) =>
                            // prettier-ignore
                            `COALESCE(` +
                                `MAX(generator${id}.heat_wh)-` +
                                `MIN(generator${id}.heat_wh)` +
                            `, 0)`
                    )
                    .join("+"),
                `ds_heat_wh`
            );
        }
    }

    static pds_energy(ids: number[]): Field {
        if (ids.length === 1) {
            return new Field(`generator${ids[0]}.energy_wh`, `s_energy`);
        } else {
            return new Field(
                ids
                    .map((id) => `COALESCE(generator${id}.energy_wh, 0)`)
                    .join("+"),
                `s_energy`
            );
        }
    }

    constructor(id: number) {
        super();
        this.name_ = `generator${id}`;
        this.from_ = new Fragment("generators");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(GeneratorSeries.time);
        return this;
    }

    public heat(alias: string | null): this {
        this.fields_.push(GeneratorSeries.heat.with_alias(alias));
        return this;
    }

    public heat_wh(alias: string | null): this {
        this.fields_.push(GeneratorSeries.heat_wh.with_alias(alias));
        return this;
    }

    public d_heat(alias: string | null): this {
        this.fields_.push(GeneratorSeries.d_heat.with_alias(alias));
        return this;
    }

    public d_energy(alias: string | null): this {
        this.fields_.push(GeneratorSeries.d_energy.with_alias(alias));
        return this;
    }

    public power(alias: string | null): this {
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
    static query_heat(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new GeneratorSeries(id)
                .time()
                .heat(`\"generator.heat_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new GeneratorSeries(id).time().heat(null).time_filter()
                    )
                )
                .fields([
                    GeneratorSeries.time,
                    new Field(`\"generator.heat_w\"`, null),
                ])
                .from(new GeneratorProxy(ids, [GeneratorSeries.heat]))
                .time_not_null()
                .ordered();
        }
    }

    static query_heat_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new GeneratorSeries(id)
                .time()
                .heat(`\"generator.heat_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new GeneratorSeries(id).time().heat(null).time_filter()
                    )
                )
                .fields([
                    GeneratorSeries.time,
                    new Field(`\"generator.heat_w\"`, null),
                ])
                .from(
                    new AggregateProxy(GeneratorSeries, ids, [
                        GeneratorSeries.ps_heat(ids).with_alias(
                            `\"generator.heat_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_power_sum(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new GeneratorSeries(id)
                .time()
                .power(`\"generator.power_w\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new GeneratorSeries(id).time().power(null).time_filter()
                    )
                )
                .fields([
                    GeneratorSeries.time,
                    new Field(`\"generator.power_w\"`, null),
                ])
                .from(
                    new AggregateProxy(GeneratorSeries, ids, [
                        GeneratorSeries.ps_power(ids).with_alias(
                            `\"generator.power_w\"`
                        ),
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }

    static query_energy_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new GeneratorSeries(ids[0])
                .d_energy(`\"generator.energy_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new GeneratorSeries(id)
                            .time()
                            .d_energy(null)
                            .time_filter()
                    )
                )
                .fields([
                    GeneratorSeries.pds_energy(ids).with_alias(
                        `\"generator.energy_wh\"`
                    ),
                ])
                .from(new Fragment(`generator${ids[0]}`))
                .joins(
                    GeneratorSeries.time_join(
                        `generator${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }

    static query_dheat_sum(ids: number[]): Query {
        if (ids.length === 1) {
            return new GeneratorSeries(ids[0])
                .d_heat(`\"generator.heat_wh\"`)
                .time_filter();
        } else {
            return new Query()
                .subqueries(
                    ids.map((id) =>
                        new GeneratorSeries(id)
                            .time()
                            .heat_wh(null)
                            .time_filter()
                    )
                )
                .fields([
                    GeneratorSeries.pds_heat(ids).with_alias(
                        `\"generator.heat_wh\"`
                    ),
                ])
                .from(new Fragment(`generator${ids[0]}`))
                .joins(
                    GeneratorSeries.time_join(
                        `generator${ids[0]}`,
                        ids.slice(1)
                    )
                );
        }
    }
}
