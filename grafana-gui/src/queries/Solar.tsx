import { Field, Fragment, Join, Query } from "./Query";
import { TimeProxy, TimeseriesProxy } from "./Proxy";
import { Samples } from "./Samples";
import { SimpleMeter, SimpleMeterSeries } from "./SimpleMeter";

export class SolarSeries extends SimpleMeterSeries {
    static basename = "solar";

    constructor(id: number) {
        super(id);
        this.name_ = `${SolarSeries.basename}${id}`;
    }
}

export class SolarProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(SolarSeries, ids, fields);
    }
}

export class Solar extends SimpleMeter {
    protected static series = SolarSeries;
    protected static proxy = SolarProxy;

    static query_energy_mon(ids: number[]): Query {
        let solar_query = null;
        if (ids.length === 1) {
            let id = ids[0];
            solar_query = new this.series(id).time().energy();
        } else {
            solar_query = new Query()
                .subqueries(
                    ids.map((id) => new this.series(id).time().energy())
                )
                .fields([
                    new TimeProxy([`${this.series.basename}${ids[0]}.time`]),
                    this.series.ps_energy(ids).with_alias("energy_wh"),
                ])
                .joins(
                    this.series.time_join(
                        `${this.series.basename}${ids[0]}`,
                        ids.slice(1)
                    )
                )
                .from(new Fragment(`${this.series.basename}${ids[0]}`));
        }

        return (
            new Query()
                .subqueries([
                    new Samples("MONTH", "1 MONTH", "12 HOUR", true),
                    solar_query.name("solar"),
                ])
                .fields([
                    new Field("samples.start", "month"),
                    // TODO: extract this
                    new Field(
                        "solar_next.energy_wh - solar_start.energy_wh",
                        "energy_wh"
                    ),
                ])
                .from(new Fragment("samples"))
                // TODO: extract this
                .joins([
                    new Join(
                        "LEFT OUTER",
                        "solar AS solar_next",
                        "solar_next.time = samples.next"
                    ),
                    new Join(
                        "LEFT OUTER",
                        "solar AS solar_start",
                        "solar_start.time = samples.start"
                    ),
                ])
        );
    }
}
