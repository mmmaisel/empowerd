import { Field } from "./Query";
import { TimeseriesProxy } from "./Proxy";
import { BidirMeter, BidirMeterSeries } from "./BidirMeter";

export class MeterSeries extends BidirMeterSeries {
    static basename = "meter";

    constructor(id: number) {
        super(id);
        this.name_ = `${MeterSeries.basename}${id}`;
    }
}

export class MeterProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(MeterSeries, ids, fields);
    }
}

export class Meter extends BidirMeter {
    protected static series = MeterSeries;
    protected static proxy = MeterProxy;
}
