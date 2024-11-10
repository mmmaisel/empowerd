import { Field, Fragment, Query, Timeseries } from "./Query";
import { ProxyQuery, TimeProxy } from "./Proxy";
import { Generator, GeneratorSeries } from "./Generator";
import { Heatpump, HeatpumpSeries } from "./Heatpump";

export type HeatingConfig = {
    generators: number[];
    heatpumps: number[];
};

export class HeatingSeries extends Timeseries {
    static basename = "heating";
    static time = new Field("time", null);

    static ps_heat(config: HeatingConfig): Field {
        if (config.generators.length === 0) {
            return HeatpumpSeries.ps_heat(config.heatpumps);
        } else if (config.heatpumps.length === 0) {
            return GeneratorSeries.ps_heat(config.generators);
        }

        return new Field(
            [
                ...config.generators.map(
                    (id) => `COALESCE(generator${id}.heat_w, 0)`
                ),
                ...config.heatpumps.map(
                    (id) => `COALESCE(heatpump${id}.heat_w, 0)`
                ),
            ].join("+"),
            `s_heat`
        );
    }
}

export class Heating {
    static query_heat_sum(config: HeatingConfig): Query {
        if (config.generators.length === 0) {
            return Heatpump.query_heat_sum(config.heatpumps);
        } else if (config.heatpumps.length === 0) {
            return Generator.query_heat_sum(config.generators);
        }

        const generators = config.generators.map((id) =>
            new GeneratorSeries(id).time().heat(null).time_filter()
        );
        const heatpumps = config.heatpumps.map((id) =>
            new HeatpumpSeries(id).time().heat(null).time_filter()
        );

        let first = "";
        let generator_ids = [...config.generators];
        let heatpump_ids = [...config.heatpumps];
        if (generator_ids.length !== 0) {
            first = `generator${generator_ids[0]}`;
            generator_ids.shift();
        } else if (heatpump_ids.length !== 0) {
            first = `heatpump${heatpump_ids[0]}`;
            heatpump_ids.shift();
        }

        const fields = [new TimeProxy(first), HeatingSeries.ps_heat(config)];

        return new Timeseries()
            .subqueries([...generators, ...heatpumps])
            .fields([new Field(`time`, null), new Field(`s_heat`, null)])
            .from(
                new ProxyQuery()
                    .fields(fields)
                    .from(new Fragment(first))
                    .joins(
                        [
                            GeneratorSeries.time_join(first, generator_ids),
                            HeatpumpSeries.time_join(first, heatpump_ids),
                        ].flat()
                    )
            )
            .time_not_null()
            .ordered();
    }
}
