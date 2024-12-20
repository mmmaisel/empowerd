import { Field } from "./Query";
import { TimeseriesProxy } from "./Proxy";
import { SimpleMeter, SimpleMeterSeries } from "./SimpleMeter";

export class WallboxSeries extends SimpleMeterSeries {
    static basename = "wallbox";

    constructor(id: number) {
        super(id);
        this.name_ = `${WallboxSeries.basename}${id}`;
    }
}

export class WallboxProxy extends TimeseriesProxy {
    constructor(ids: number[], fields: Field[]) {
        super(WallboxSeries, ids, fields);
    }
}

export class Wallbox extends SimpleMeter {
    protected static series = WallboxSeries;
    protected static proxy = WallboxProxy;
}
