import { Field, Fragment, Query, Timeseries } from "./Query";

export class WeatherSeries extends Timeseries {
    static basename = "weather";
    static time = new Field("time", null);
    static temp_in = new Field("temp_in_degc_e1/10.0", "temp_in_degc");
    static hum_in = new Field("hum_in_e3/10.0", "hum_in_pct");
    static temp_out = new Field("temp_out_degc_e1/10.0", "temp_out_degc");
    static hum_out = new Field("hum_out_e3/10.0", "hum_out_pct");
    static dew_point = new Field("dew_point_degc_e1/10.0", "dew_point_degc");
    static temp_x1 = new Field("temp_x1_degc_e1/10.0", "temp_x1_degc");
    static hum_x1 = new Field("hum_x1_e3/10.0", "hum_x1_pct");
    static temp_x2 = new Field("temp_x2_degc_e1/10.0", "temp_x2_degc");
    static hum_x2 = new Field("hum_x2_e3/10.0", "hum_x2_pct");
    static temp_x3 = new Field("temp_x3_degc_e1/10.0", "temp_x3_degc");
    static hum_x3 = new Field("hum_x3_e3/10.0", "hum_x3_pct");
    static rain_act = new Field("rain_act_um", null);

    constructor(id: number) {
        super();
        this.name_ = `weather${id}`;
        this.from_ = new Fragment("weathers");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(WeatherSeries.time);
        return this;
    }

    public temp_in(alias: string | null): this {
        this.fields_.push(WeatherSeries.temp_in.with_alias(alias));
        return this;
    }

    public hum_in(alias: string | null): this {
        this.fields_.push(WeatherSeries.hum_in.with_alias(alias));
        return this;
    }

    public temp_out(alias: string | null): this {
        this.fields_.push(WeatherSeries.temp_out.with_alias(alias));
        return this;
    }

    public hum_out(alias: string | null): this {
        this.fields_.push(WeatherSeries.hum_out.with_alias(alias));
        return this;
    }

    public dew_point(alias: string | null): this {
        this.fields_.push(WeatherSeries.dew_point.with_alias(alias));
        return this;
    }

    public temp_x1(alias: string | null): this {
        this.fields_.push(WeatherSeries.temp_x1.with_alias(alias));
        return this;
    }

    public hum_x1(alias: string | null): this {
        this.fields_.push(WeatherSeries.hum_x1.with_alias(alias));
        return this;
    }

    public temp_x2(alias: string | null): this {
        this.fields_.push(WeatherSeries.temp_x2.with_alias(alias));
        return this;
    }

    public hum_x2(alias: string | null): this {
        this.fields_.push(WeatherSeries.hum_x2.with_alias(alias));
        return this;
    }

    public temp_x3(alias: string | null): this {
        this.fields_.push(WeatherSeries.temp_x3.with_alias(alias));
        return this;
    }

    public hum_x3(alias: string | null): this {
        this.fields_.push(WeatherSeries.hum_x3.with_alias(alias));
        return this;
    }

    public rain_act(alias: string | null): this {
        this.fields_.push(WeatherSeries.rain_act.with_alias(alias));
        return this;
    }
}

export class Weather {
    static query_temp_rain(ids: number[]): Query {
        return new WeatherSeries(ids[0])
            .time()
            .temp_out(null)
            .rain_act(null)
            .time_filter()
            .ordered();
    }

    static query_temps(ids: number[]): Query {
        return new WeatherSeries(ids[0])
            .time()
            .temp_in(null)
            .temp_out(null)
            .dew_point(null)
            .temp_x1(null)
            .temp_x2(null)
            .temp_x3(null)
            .time_filter()
            .ordered();
    }

    static query_hums(ids: number[]): Query {
        return new WeatherSeries(ids[0])
            .time()
            .hum_in(null)
            .hum_out(null)
            .hum_x1(null)
            .hum_x2(null)
            .hum_x3(null)
            .time_filter()
            .ordered();
    }
}
