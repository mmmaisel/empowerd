import { Field, Fragment, Join, Query, Timeseries } from "./Query";
import { Samples } from "./Samples";

export class WeatherSeries extends Timeseries {
    static basename = "weather";
    static time = new Field("time");
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
    static temp_x4 = new Field("temp_x4_degc_e1/10.0", "temp_x4_degc");
    static hum_x4 = new Field("hum_x4_e3/10.0", "hum_x4_pct");
    static temp_x5 = new Field("temp_x5_degc_e1/10.0", "temp_x5_degc");
    static hum_x5 = new Field("hum_x5_e3/10.0", "hum_x5_pct");
    static temp_x6 = new Field("temp_x6_degc_e1/10.0", "temp_x6_degc");
    static hum_x6 = new Field("hum_x6_e3/10.0", "hum_x6_pct");
    static temp_x7 = new Field("temp_x7_degc_e1/10.0", "temp_x7_degc");
    static hum_x7 = new Field("hum_x7_e3/10.0", "hum_x7_pct");
    static rain_act = new Field("rain_act_um/1000.0", "rain_act_mm");
    static rain_day = new Field("rain_day_um/1000.0", "rain_day_mm");
    static baro_abs = new Field("baro_abs_pa/100.0", "baro_abs_hpa");
    static baro_sea = new Field("baro_sea_pa/100.0", "baro_sea_hpa");
    static wind_act = new Field("wind_act_mms/1000.0", "wind_act_ms");
    static wind_gust = new Field("wind_gust_mms/1000.0", "wind_gust_ms");
    static wind_dir = new Field("wind_dir_deg_e1/10.0", "wind_dir_deg");

    constructor(id: number) {
        super();
        this.name_ = `${WeatherSeries.basename}${id}`;
        this.from_ = new Fragment("weathers");
        this.wheres_ = [`series_id = ${id}`];
    }

    public time(): this {
        this.fields_.push(WeatherSeries.time);
        return this;
    }

    public temp_in(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_in.with_alias(alias));
        return this;
    }

    public hum_in(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_in.with_alias(alias));
        return this;
    }

    public temp_out(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_out.with_alias(alias));
        return this;
    }

    public hum_out(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_out.with_alias(alias));
        return this;
    }

    public dew_point(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.dew_point.with_alias(alias));
        return this;
    }

    public temp_x1(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x1.with_alias(alias));
        return this;
    }

    public hum_x1(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x1.with_alias(alias));
        return this;
    }

    public temp_x2(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x2.with_alias(alias));
        return this;
    }

    public hum_x2(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x2.with_alias(alias));
        return this;
    }

    public temp_x3(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x3.with_alias(alias));
        return this;
    }

    public hum_x3(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x3.with_alias(alias));
        return this;
    }

    public temp_x4(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x4.with_alias(alias));
        return this;
    }

    public hum_x4(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x4.with_alias(alias));
        return this;
    }

    public temp_x5(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x5.with_alias(alias));
        return this;
    }

    public hum_x5(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x5.with_alias(alias));
        return this;
    }

    public temp_x6(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x6.with_alias(alias));
        return this;
    }

    public hum_x6(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x6.with_alias(alias));
        return this;
    }

    public temp_x7(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.temp_x7.with_alias(alias));
        return this;
    }

    public hum_x7(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.hum_x7.with_alias(alias));
        return this;
    }

    public rain_act(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.rain_act.with_alias(alias));
        return this;
    }

    public rain_day(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.rain_day.with_alias(alias));
        return this;
    }

    public baro_abs(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.baro_abs.with_alias(alias));
        return this;
    }

    public baro_sea(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.baro_sea.with_alias(alias));
        return this;
    }

    public wind_act(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.wind_act.with_alias(alias));
        return this;
    }

    public wind_gust(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.wind_gust.with_alias(alias));
        return this;
    }

    public wind_dir(alias: string | null = null): this {
        this.fields_.push(WeatherSeries.wind_dir.with_alias(alias));
        return this;
    }
}

export class Weather {
    protected static series = WeatherSeries;

    static query_temp_rain(ids: number[]): Query {
        return new this.series(ids[0])
            .time()
            .temp_out()
            .rain_act()
            .time_filter()
            .ordered();
    }

    static query_temps(ids: number[]): Query {
        return new this.series(ids[0])
            .time()
            .temp_in()
            .temp_out()
            .dew_point()
            .temp_x1()
            .temp_x2()
            .temp_x3()
            .temp_x4()
            .temp_x5()
            .temp_x6()
            .temp_x7()
            .time_filter()
            .ordered();
    }

    static query_hums(ids: number[]): Query {
        return new this.series(ids[0])
            .time()
            .hum_in()
            .hum_out()
            .hum_x1()
            .hum_x2()
            .hum_x3()
            .hum_x4()
            .hum_x5()
            .hum_x6()
            .hum_x7()
            .time_filter()
            .ordered();
    }

    static query_rain(ids: number[]): Query {
        return new this.series(ids[0])
            .time()
            .rain_act()
            .rain_day()
            .time_filter()
            .ordered();
    }

    static query_baro(ids: number[]): Query {
        return new this.series(ids[0])
            .time()
            .baro_abs()
            .baro_sea()
            .time_filter()
            .ordered();
    }

    static query_wind(ids: number[]): Query {
        return new this.series(ids[0])
            .time()
            .wind_act()
            .wind_gust()
            .wind_dir()
            .time_filter()
            .ordered();
    }

    static query_rain_int(ids: number[]): Query {
        let id = ids[0];
        let rain_query = new this.series(id).time().rain_day();

        // TODO: samples are buggy because rain reset is in local time
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
