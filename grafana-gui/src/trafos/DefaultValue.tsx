import { DataFrame } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

export const DefaultValueTrafo =
    (...args: any[]) =>
    (source: Observable<DataFrame[]>): Observable<DataFrame[]> => {
        return source.pipe(
            map((data: DataFrame[]) => {
                for (let frame of data) {
                    if (frame.length === 0) {
                        frame.length = 1;
                        frame.fields = [
                            {
                                config: {},
                                name: "time",
                                type: "time" as any,
                                values: [null],
                            },
                            {
                                config: {},
                                name: "value",
                                type: "number" as any,
                                values: [null],
                            },
                        ];
                    }
                }
                return data;
            })
        );
    };
