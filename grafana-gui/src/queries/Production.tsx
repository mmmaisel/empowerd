import { Field, Fragment, Query, Timeseries } from "./Query";
import { TimeProxy, ProxyQuery } from "./Proxy";
import { Generator, GeneratorSeries } from "./Generator";
import { Solar, SolarSeries } from "./Solar";

export type ProductionConfig = {
    generators: number[];
    solars: number[];
};

export class ProductionSeries extends Timeseries {
    static basename = "production";
    static time = new Field("time");

    static ps_power(config: ProductionConfig): Field {
        if (config.solars.length === 0) {
            return GeneratorSeries.ps_power(config.generators);
        } else if (config.generators.length === 0) {
            return SolarSeries.ps_power(config.solars);
        }

        return new Field(
            [
                ...config.solars.map((id) => `COALESCE(solar${id}.power_w, 0)`),
                ...config.generators.map(
                    (id) => `COALESCE(generator${id}.power_w, 0)`
                ),
            ].join("+"),
            `s_power`
        );
    }
}

export class Production {
    protected static series = ProductionSeries;

    static query_power_sum(config: ProductionConfig): Query {
        if (config.solars.length === 0) {
            return Generator.query_power_sum(config.generators);
        } else if (config.generators.length === 0) {
            return Solar.query_power_sum(config.solars);
        }

        const solars = config.solars.map((id) =>
            new SolarSeries(id).time().power().time_filter()
        );
        const generators = config.generators.map((id) =>
            new GeneratorSeries(id).time().power().time_filter()
        );

        let first = "";
        let solar_ids = [...config.solars];
        let generator_ids = [...config.generators];
        if (solar_ids.length !== 0) {
            first = `solar${solar_ids[0]}`;
            solar_ids.shift();
        } else if (generator_ids.length !== 0) {
            first = `generator${generator_ids[0]}`;
            generator_ids.shift();
        }

        const fields = [
            TimeProxy.from_series([...generators, ...solars]),
            this.series
                .ps_power(config)
                .with_alias(`\"${this.series.basename}.power_w\"`),
        ];

        return new Timeseries()
            .subqueries([...solars, ...generators])
            .fields([
                new Field(`time`),
                new Field(`\"${this.series.basename}.power_w\"`),
            ])
            .from(
                new ProxyQuery()
                    .fields(fields)
                    .from(new Fragment(first))
                    .joins(
                        [
                            SolarSeries.time_join(first, solar_ids),
                            GeneratorSeries.time_join(first, generator_ids),
                        ].flat()
                    )
            )
            .time_not_null()
            .ordered();
    }
}
