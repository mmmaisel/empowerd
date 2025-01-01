export class Color {
    public red: number;
    public green: number;
    public blue: number;
    public alpha: number;

    constructor(red: number, green: number, blue: number, alpha: number) {
        this.red = red;
        this.green = green;
        this.blue = blue;
        this.alpha = alpha;
    }

    public static from_hsl(h: number, s: number, l: number): Color {
        const c = (1 - Math.abs(l / 127 - 1)) * s;
        const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
        const m = l - c / 2;

        if (h < 60) {
            return new Color(c + m, x + m, m, 1);
        } else if (h < 120) {
            return new Color(x + m, c + m, m, 1);
        } else if (h < 180) {
            return new Color(m, c + m, x + m, 1);
        } else if (h < 240) {
            return new Color(m, x + m, c + m, 1);
        } else if (h < 300) {
            return new Color(x + m, m, c + m, 1);
        } else {
            return new Color(c + m, m, x + m, 1);
        }
    }

    public with_alpha(alpha: number): this {
        this.alpha = alpha;
        return this;
    }

    public to_rgb(): string {
        return `rgb(${this.red}, ${this.green}, ${this.blue})`;
    }

    public to_rgba(): string {
        return `rgba(${this.red}, ${this.green}, ${this.blue}, ${this.alpha})`;
    }

    public static red = (i: number): Color => {
        const colors = [
            Color.from_hsl(0, 206, 110),
            Color.from_hsl(340, 206, 110),
            Color.from_hsl(20, 206, 110),
            Color.from_hsl(0, 206, 150),
            Color.from_hsl(340, 206, 150),
            Color.from_hsl(20, 206, 150),
            Color.from_hsl(0, 206, 90),
            Color.from_hsl(340, 206, 90),
            Color.from_hsl(20, 206, 90),
        ];

        return colors[i % colors.length];
    };

    public static orange = (i: number): Color => {
        const colors = [
            Color.from_hsl(35, 206, 130),
            Color.from_hsl(40, 206, 130),
            Color.from_hsl(30, 206, 130),
            Color.from_hsl(35, 206, 160),
            Color.from_hsl(40, 206, 160),
            Color.from_hsl(30, 206, 160),
            Color.from_hsl(35, 206, 100),
            Color.from_hsl(40, 206, 100),
            Color.from_hsl(30, 206, 100),
        ];

        return colors[i % colors.length];
    };

    public static yellow = (i: number): Color => {
        const colors = [
            Color.from_hsl(50, 230, 128),
            Color.from_hsl(40, 230, 128),
            Color.from_hsl(60, 230, 128),
            Color.from_hsl(50, 230, 160),
            Color.from_hsl(40, 230, 160),
            Color.from_hsl(60, 230, 160),
            Color.from_hsl(50, 230, 96),
            Color.from_hsl(40, 230, 96),
            Color.from_hsl(60, 230, 96),
        ];

        return colors[i % colors.length];
    };

    public static green = (i: number): Color => {
        const colors = [
            Color.from_hsl(115, 128, 90),
            Color.from_hsl(100, 128, 90),
            Color.from_hsl(130, 128, 90),
            Color.from_hsl(115, 128, 120),
            Color.from_hsl(100, 128, 120),
            Color.from_hsl(130, 128, 120),
            Color.from_hsl(115, 128, 60),
            Color.from_hsl(100, 128, 60),
            Color.from_hsl(130, 128, 60),
        ];

        return colors[i % colors.length];
    };

    public static cyan = (i: number): Color => {
        const colors = [
            Color.from_hsl(185, 173, 130),
            Color.from_hsl(175, 173, 130),
            Color.from_hsl(195, 173, 130),
            Color.from_hsl(185, 173, 160),
            Color.from_hsl(175, 173, 160),
            Color.from_hsl(195, 173, 160),
            Color.from_hsl(185, 173, 100),
            Color.from_hsl(175, 173, 100),
            Color.from_hsl(195, 173, 100),
        ];

        return colors[i % colors.length];
    };

    public static blue = (i: number): Color => {
        const colors = [
            Color.from_hsl(220, 186, 110),
            Color.from_hsl(210, 186, 110),
            Color.from_hsl(230, 186, 110),
            Color.from_hsl(220, 186, 150),
            Color.from_hsl(210, 186, 150),
            Color.from_hsl(230, 186, 150),
            Color.from_hsl(220, 186, 90),
            Color.from_hsl(210, 186, 90),
            Color.from_hsl(230, 186, 90),
        ];

        return colors[i % colors.length];
    };

    public static purple = (i: number): Color => {
        const colors = [
            Color.from_hsl(280, 130, 120),
            Color.from_hsl(270, 130, 120),
            Color.from_hsl(290, 130, 120),
            Color.from_hsl(280, 130, 150),
            Color.from_hsl(270, 130, 150),
            Color.from_hsl(290, 130, 150),
            Color.from_hsl(280, 130, 90),
            Color.from_hsl(270, 130, 90),
            Color.from_hsl(290, 130, 90),
        ];

        return colors[i % colors.length];
    };

    public static grey = (i: number): Color => {
        const colors = [
            Color.from_hsl(220, 255, 220),
            Color.from_hsl(210, 255, 220),
            Color.from_hsl(230, 255, 220),
            Color.from_hsl(220, 255, 250),
            Color.from_hsl(210, 255, 250),
            Color.from_hsl(230, 255, 250),
            Color.from_hsl(220, 255, 190),
            Color.from_hsl(210, 255, 190),
            Color.from_hsl(230, 255, 190),
        ];

        return colors[i % colors.length];
    };
}
