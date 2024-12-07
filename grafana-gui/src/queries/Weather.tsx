import { Field, Fragment, Join, Query, Timeseries } from "./Query";
import { Samples } from "./Samples";

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
    static rain_act = new Field("rain_act_um/1000.0", "rain_act_mm");
    static rain_day = new Field("rain_day_um/1000.0", "rain_day_mm");
    static baro_abs = new Field("baro_abs_pa/100.0", "baro_abs_hpa");
    static baro_sea = new Field("baro_sea_pa/100.0", "baro_sea_hpa");
    static wind_act = new Field("wind_act_mms/1000.0", "wind_act_ms");
    static wind_gust = new Field("wind_gust_mms/1000.0", "wind_gust_ms");
    static wind_dir = new Field("wind_dir_deg_e1/10.0", "wind_dir_deg");

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

    public rain_day(alias: string | null): this {
        this.fields_.push(WeatherSeries.rain_day.with_alias(alias));
        return this;
    }

    public baro_abs(alias: string | null): this {
        this.fields_.push(WeatherSeries.baro_abs.with_alias(alias));
        return this;
    }

    public baro_sea(alias: string | null): this {
        this.fields_.push(WeatherSeries.baro_sea.with_alias(alias));
        return this;
    }

    public wind_act(alias: string | null): this {
        this.fields_.push(WeatherSeries.wind_act.with_alias(alias));
        return this;
    }

    public wind_gust(alias: string | null): this {
        this.fields_.push(WeatherSeries.wind_gust.with_alias(alias));
        return this;
    }

    public wind_dir(alias: string | null): this {
        this.fields_.push(WeatherSeries.wind_dir.with_alias(alias));
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

    static query_rain(ids: number[]): Query {
        return new WeatherSeries(ids[0])
            .time()
            .rain_act(null)
            .rain_day(null)
            .time_filter()
            .ordered();
    }

    static query_baro(ids: number[]): Query {
        return new WeatherSeries(ids[0])
            .time()
            .baro_abs(null)
            .baro_sea(null)
            .time_filter()
            .ordered();
    }

    static query_wind(ids: number[]): Query {
        return new WeatherSeries(ids[0])
            .time()
            .wind_act(null)
            .wind_gust(null)
            .wind_dir(null)
            .time_filter()
            .ordered();
    }

    static query_rain_int(ids: number[]): Query {
        let id = ids[0];
        let rain_query = new WeatherSeries(id).time().rain_day(null);

        return new Query()
            .subqueries([
                new Samples("DAY", "1 DAY", "23:00", false),
                rain_query.name("weather"),
            ])
            .fields([new Field("SUM(weather.rain_day_mm)", "rain_int_mm")])
            .from(new Fragment("samples"))
            .joins([
                new Join(
                    "LEFT OUTER",
                    "weather",
                    "weather.time = samples.start"
                ),
            ]);
    }
}