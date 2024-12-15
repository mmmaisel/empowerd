import { Field, Fragment, Query, Timeseries } from "./Query";
import { TimeseriesProxy } from "./Proxy";

export class BoilerSeries extends Timeseries {
    static basename = "boiler";
    static time = new Field("time");
    static top = new Field("boiler_top_degc_e1 / 10.0", "top");
    static mid = new Field("boiler_mid_degc_e1 / 10.0", "mid");
    static bot = new Field("boiler_bot_degc_e1 / 10.0", "bot");

    constructor(id: number) {
        super();
        this.name_ = `${BoilerSeries.basename}${id}`;
        this.from_ = new Fragment("heatpumps");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(BoilerSeries.time);
        return this;
    }

    public top(alias: string | null = null): this {
        this.fields_.push(BoilerSeries.top.with_alias(alias));
        return this;
    }

    public mid(alias: string | null = null): this {
        this.fields_.push(BoilerSeries.mid.with_alias(alias));
        return this;
    }

    public bot(alias: string | null = null): this {
        this.fields_.push(BoilerSeries.bot.with_alias(alias));
        return this;
    }
}

export class BoilerProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(BoilerSeries, ids, fields);
    }
}

export class Boiler {
    protected static series = BoilerSeries;

    static query_temps(ids: number[]): Query {
        if (ids.length === 1) {
            let id = ids[0];
            return new BoilerSeries(id)
                .time()
                .top(`\"${this.series.basename}${id}.top\"`)
                .mid(`\"${this.series.basename}${id}.mid\"`)
                .bot(`\"${this.series.basename}${id}.bot\"`)
                .time_filter()
                .ordered();
        } else {
            return new Timeseries()
                .subqueries(
                    ids.map((id) =>
                        new BoilerSeries(id)
                            .time()
                            .top()
                            .mid()
                            .bot()
                            .time_filter()
                    )
                )
                .fields(
                    [
                        BoilerSeries.time,
                        ids
                            .map((id) => [
                                new Field(
                                    `\"${this.series.basename}${id}.top\"`
                                ),
                                new Field(
                                    `\"${this.series.basename}${id}.mid\"`
                                ),
                                new Field(
                                    `\"${this.series.basename}${id}.bot\"`
                                ),
                            ])
                            .flat(),
                    ].flat()
                )
                .from(
                    new BoilerProxy(ids, [
                        BoilerSeries.top,
                        BoilerSeries.mid,
                        BoilerSeries.bot,
                    ])
                )
                .time_not_null()
                .ordered();
        }
    }
}
