import { DataFrame } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

const b = 18.678;
const c = 257.14; // degC
const d = 234.5; // degC
const RD = 461.51; // J/kg/K
const absZero = -273.15;

const f_gamma_m = function (rh: number, temp: number): number {
    return Math.log(
        (rh / 100) * Math.exp((b - temp / d) * (temp / (c + temp)))
    );
};

const f_esat_w = function (temp: number): number {
    return 611.2 * Math.exp((17.62 * temp) / (243.12 + temp));
};

export const RoomDetailsTrafo =
    (...args: any[]) =>
    (source: Observable<DataFrame[]>): Observable<DataFrame[]> => {
        return source.pipe(
            map((data: DataFrame[]) => {
                if (data.length === 0) {
                    return [];
                }
                if (data[0].fields.length !== 3) {
                    return [];
                }

                // Override source names for later matching
                data[0].fields[1].name = "temp_degc";
                data[0].fields[2].name = "humidity_pct";

                let dew_points = {
                    name: `dew_point_degc`,
                    type: "number" as any,
                    config: {},
                    values: new Array(data[0].length),
                };
                let abs_humidities = {
                    name: `humidity_kgm3`,
                    type: "number" as any,
                    config: {},
                    values: new Array(data[0].length),
                };

                for (let i = 0; i < data[0].length; ++i) {
                    let temp = data[0].fields[1].values[i];
                    let rel_hum = data[0].fields[2].values[i];

                    let gamma_m = f_gamma_m(rel_hum, temp);
                    dew_points.values[i] = (c * gamma_m) / (b - gamma_m);
                    // 1000 * rel_hum / 100
                    abs_humidities.values[i] =
                        (10 * f_esat_w(temp) * rel_hum) / RD / (temp - absZero);
                }

                return [
                    {
                        fields: [...data[0].fields, dew_points, abs_humidities],
                        length: 5,
                        refId: "A",
                    },
                ];
            })
        );
    };
