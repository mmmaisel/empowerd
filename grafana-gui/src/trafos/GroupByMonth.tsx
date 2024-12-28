import { DataFrame } from "@grafana/data";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

const month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

export const GroupByMonthTrafo =
    (name: string, unit: string, palette: any, ...args: any[]) =>
    (source: Observable<DataFrame[]>): Observable<DataFrame[]> => {
        return source.pipe(
            map((data: DataFrame[]) => {
                if (data.length === 0) {
                    return [];
                }

                let av_months: Set<number> = new Set();
                let av_years: Set<number> = new Set();
                for (let timestamp of data[0].fields[0].values) {
                    let date = new Date(timestamp);
                    av_months.add(date.getMonth());
                    av_years.add(date.getFullYear());
                }

                const first_month = [...av_months][0];
                const first_year = Math.min(...av_years);
                let months = {
                    name: "month",
                    type: "string" as any,
                    config: {},
                    values: [...av_months].map((x: number) => month_names[x]),
                };
                let years: any[] = [...av_years].map((x: number) => {
                    return {
                        name: `${name} ${x}`,
                        type: "number" as any,
                        config: {
                            color: {
                                fixedColor: palette(x - first_year).to_rgb(),
                                mode: "fixed",
                            },
                            unit: unit,
                        },
                        values: Array(12).fill(null),
                    };
                });

                for (let i = 0; i < data[0].fields[0].values.length; ++i) {
                    let date = new Date(data[0].fields[0].values[i]);
                    years[date.getFullYear() - first_year].values[
                        (date.getMonth() - first_month + 12) % 12
                    ] = data[0].fields[1].values[i];
                }

                // TODO: always move Jan to front

                return [
                    {
                        fields: [months, ...years],
                        length: years.length + 1,
                        refId: "A",
                    },
                ];
            })
        );
    };
