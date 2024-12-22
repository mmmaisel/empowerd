import { Field, Fragment, Query, Timeseries } from "./Query";
import { TimeProxy, ProxyQuery } from "./Proxy";
import { BatterySeries } from "./Battery";
import { GeneratorSeries } from "./Generator";
import { HeatpumpSeries } from "./Heatpump";
import { MeterSeries } from "./Meter";
import { SolarSeries } from "./Solar";
import { WallboxSeries } from "./Wallbox";

export type ConsumptionConfig = {
    batteries: number[];
    generators: number[];
    heatpumps: number[];
    meters: number[];
    solars: number[];
    wallboxes: number[];
};

export class ConsumptionSeries extends Timeseries {
    static basename = "consumption";
    static time = new Field("time");

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
            (id) => `COALESCE(battery${id}.npower_w, 0)`
        );
        let generators = config.generators.map(
            (id) => `COALESCE(generator${id}.power_w, 0)`
        );
        let heatpumps = config.heatpumps.map(
            (id) => `COALESCE(heatpump${id}.power_w, 0)`
        );
        let solars = config.solars.map(
            (id) => `COALESCE(solar${id}.power_w, 0)`
        );
        let wallboxes = config.wallboxes.map(
            (id) => `COALESCE(wallbox${id}.power_w, 0)`
        );

        return new Field(
            [
                [...meters, ...batteries, ...generators, ...solars].join("+"),
                [...heatpumps, ...wallboxes].join("-"),
            ]
                .filter((str) => str.length != 0)
                .join("-"),
            `s_power`
        );
    }

    static pds_energy(config: ConsumptionConfig): Field {
        if (config.meters.length === 0) {
            return new Field("NULL", "s_power");
        }

        // TODO: reuse pds_energy proxies
        let meters = config.meters.map(
            (id) =>
                `COALESCE(` +
                `MAX(meter${id}.energy_in_wh)-MIN(meter${id}.energy_in_wh)` +
                `-MAX(meter${id}.energy_out_wh)+MIN(meter${id}.energy_out_wh)` +
                `, 0)`
        );
        let batteries = config.batteries.map(
            (id) =>
                `COALESCE(` +
                `MAX(battery${id}.energy_out_wh)-MIN(battery${id}.energy_out_wh)` +
                `-MAX(battery${id}.energy_in_wh)+MIN(battery${id}.energy_in_wh)` +
                `, 0)`
        );
        let generators = config.generators.map(
            (id) =>
                `COALESCE(MAX(generator${id}.energy_wh)-MIN(generator${id}.energy_wh), 0)`
        );
        let heatpumps = config.heatpumps.map(
            (id) =>
                `COALESCE(MAX(heatpump${id}.energy_wh)-MIN(heatpump${id}.energy_wh), 0)`
        );
        let solars = config.solars.map(
            (id) =>
                `COALESCE(MAX(solar${id}.energy_wh)-MIN(solar${id}.energy_wh), 0)`
        );
        let wallboxes = config.wallboxes.map(
            (id) =>
                `COALESCE(MAX(wallbox${id}.energy_wh)-MIN(wallbox${id}.energy_wh), 0)`
        );

        return new Field(
            [
                [...meters, ...batteries, ...generators, ...solars].join("+"),
                [...heatpumps, ...wallboxes].join("-"),
            ]
                .filter((str) => str.length != 0)
                .join("-"),
            `d_energy_wh`
        );
    }
}

export class Consumption {
    protected static series = ConsumptionSeries;

    static query_power_sum(config: ConsumptionConfig): Query {
        if (config.meters.length === 0) {
            return new Timeseries().fields([new Field("NULL", "s_power")]);
        }

        const meters = config.meters.map((id) =>
            new MeterSeries(id).time().power().time_filter()
        );
        const batteries = config.batteries.map((id) =>
            new BatterySeries(id).time().npower().time_filter()
        );
        const generators = config.generators.map((id) =>
            new GeneratorSeries(id).time().power().time_filter()
        );
        const heatpumps = config.heatpumps.map((id) =>
            new HeatpumpSeries(id).time().power().time_filter()
        );
        const solars = config.solars.map((id) =>
            new SolarSeries(id).time().power().time_filter()
        );
        const wallboxes = config.wallboxes.map((id) =>
            new WallboxSeries(id).time().power().time_filter()
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
                ...heatpumps,
                ...solars,
                ...wallboxes,
            ]),
            this.series
                .ps_power(config)
                .with_alias(`\"${this.series.basename}.power_w\"`),
        ];

        return new Timeseries()
            .subqueries([
                ...meters,
                ...batteries,
                ...generators,
                ...heatpumps,
                ...solars,
                ...wallboxes,
            ])
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
                            MeterSeries.time_join(first, meter_ids),
                            BatterySeries.time_join(first, config.batteries),
                            GeneratorSeries.time_join(first, config.generators),
                            HeatpumpSeries.time_join(first, config.heatpumps),
                            SolarSeries.time_join(first, config.solars),
                            WallboxSeries.time_join(first, config.wallboxes),
                        ].flat()
                    )
            )
            .time_not_null()
            .ordered();
    }

    static query_denergy_sum(config: ConsumptionConfig): Query {
        if (config.meters.length === 0) {
            return new Timeseries().fields([
                new Field("NULL", `\"${this.series.basename}.d_energy_wh\"`),
            ]);
        }

        const meters = config.meters.map((id) =>
            new MeterSeries(id).time().energy_in().energy_out().time_filter()
        );
        const batteries = config.batteries.map((id) =>
            new BatterySeries(id).time().energy_in().energy_out().time_filter()
        );
        const generators = config.generators.map((id) =>
            new GeneratorSeries(id).time().energy().time_filter()
        );
        const heatpumps = config.heatpumps.map((id) =>
            new HeatpumpSeries(id).time().energy().time_filter()
        );
        const solars = config.solars.map((id) =>
            new SolarSeries(id).time().energy().time_filter()
        );
        const wallboxes = config.wallboxes.map((id) =>
            new WallboxSeries(id).time().energy().time_filter()
        );

        let first = "";
        let meter_ids = [...config.meters];
        first = `meter${meter_ids[0]}`;
        meter_ids.shift();

        return new Timeseries()
            .subqueries([
                ...meters,
                ...batteries,
                ...generators,
                ...heatpumps,
                ...solars,
                ...wallboxes,
            ])
            .fields([this.series.pds_energy(config)])
            .from(new Fragment(first))
            .joins(
                [
                    MeterSeries.time_join(first, meter_ids),
                    BatterySeries.time_join(first, config.batteries),
                    GeneratorSeries.time_join(first, config.generators),
                    HeatpumpSeries.time_join(first, config.heatpumps),
                    SolarSeries.time_join(first, config.solars),
                    WallboxSeries.time_join(first, config.wallboxes),
                ].flat()
            );
    }
}
