import { Query } from "./Query";

export class Samples extends Query {
    private trunc: string;
    private interval: string;
    private offset: string;
    private next: boolean;

    constructor(
        trunc: string,
        interval: string,
        offset: string,
        next: boolean
    ) {
        super();
        this.name_ = "samples";
        this.trunc = trunc;
        this.interval = interval;
        this.offset = offset;
        this.next = next;
    }

    public sql(): string {
        const samples =
            // prettier-ignore
            "points AS (" +
                "SELECT GENERATE_SERIES(" +
                    `DATE_TRUNC('${this.trunc}', TIMESTAMP $__timeFrom()),` +
                    `DATE_TRUNC('${this.trunc}', TIMESTAMP $__timeTo()),` +
                    `INTERVAL '${this.interval}'` +
                ") AS point" +
            ")";

        let selects = [`point + INTERVAL '${this.offset}' AS start`];
        if (this.next) {
            selects.push(
                `point + INTERVAL '${this.interval}' + ` +
                    `INTERVAL '${this.offset}' AS next`
            );
        }

        return `WITH ${samples} SELECT ${selects.join(", ")} FROM points`;
    }
}
