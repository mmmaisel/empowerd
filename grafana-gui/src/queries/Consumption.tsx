import { Field, Fragment, Query, Timeseries } from "./Query";
import { TimeProxy, ProxyQuery } from "./Proxy";
import { BatterySeries } from "./Battery";
import { GeneratorSeries } from "./Generator";
import { MeterSeries } from "./Meter";
import { SolarSeries } from "./Solar";

export type ConsumptionConfig = {
    batteries: number[];
    generators: number[];
    meters: number[];
    solars: number[];
};

export class ConsumptionSeries extends Timeseries {
    static basename = "consumption";
    static time = new Field("time", null);

    static ps_power(config: ConsumptionConfig): Field {
        if (config.meters.length === 0) {
            return new Field("NULL", "s_power");
        }

        // TODO: optimize for single sources?
        // TODO: move these maps to other classes
        let meters = config.meters.map(
            (id) => `COALESCE(meter${id}.power_w, 0)`
        );
        let batteries = config.batteries.map(
            (id) => `COALESCE(-battery${id}.power_w, 0)`
        );
        let generators = config.generators.map(
            (id) => `COALESCE(generator${id}.power_w, 0)`
        );
        let solars = config.solars.map(
            (id) => `COALESCE(solar${id}.power_w, 0)`
        );

        return new Field(
            [...meters, ...batteries, ...generators, ...solars].join("+"),
            `s_power`
        );
    }
}

export class Consumption {
    static query_power_sum(config: ConsumptionConfig): Query {
        if (config.meters.length === 0) {
            return new Timeseries().fields([new Field("NULL", "s_power")]);
        }

        const meters = config.meters.map((id) =>
            new MeterSeries(id).time().power(null).time_filter()
        );
        const batteries = config.batteries.map((id) =>
            new BatterySeries(id).time().power(null).time_filter()
        );
        const generators = config.generators.map((id) =>
            new GeneratorSeries(id).time().power(null).time_filter()
        );
        const solars = config.solars.map((id) =>
            new SolarSeries(id).time().power(null).time_filter()
        );

        let first = "";
        let meter_ids = [...config.meters];
        first = `meter${meter_ids[0]}`;
        meter_ids.shift();

        const fields = [
            TimeProxy.from_series([
                ...meters,
                ...batteries,
                ...generators,
                ...solars,
            ]),
            ConsumptionSeries.ps_power(config).with_alias(
                `\"consumption.power_w\"`
            ),
        ];

        return new Timeseries()
            .subqueries([...meters, ...batteries, ...generators, ...solars])
            .fields([
                new Field(`time`, null),
                new Field(`\"consumption.power_w\"`, null),
            ])
            .from(
                new ProxyQuery()
                    .fields(fields)
                    .from(new Fragment(first))
                    .joins(
                        [
                            MeterSeries.time_join(first, meter_ids),
                            BatterySeries.time_join(first, config.batteries),
                            GeneratorSeries.time_join(first, config.generators),
                            SolarSeries.time_join(first, config.solars),
                        ].flat()
                    )
            )
            .time_not_null()
            .ordered();
    }
}
