const to_02hex = (x: number): string => {
    let hex = Math.round(x).toString(16);
    if (hex.length < 2) {
        hex = `0${hex}`;
    }
    return hex;
};

const hsl_to_rgb = (h: number, s: number, l: number): string => {
    const c = (1 - Math.abs(l / 127 - 1)) * s;
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
    const m = l - c / 2;

    if (h < 60) {
        return `#${to_02hex(c + m)}${to_02hex(x + m)}${to_02hex(m)}`;
    } else if (h < 120) {
        return `#${to_02hex(x + m)}${to_02hex(c + m)}${to_02hex(m)}`;
    } else if (h < 180) {
        return `#${to_02hex(m)}${to_02hex(c + m)}${to_02hex(x + m)}`;
    } else if (h < 240) {
        return `#${to_02hex(m)}${to_02hex(x + m)}${to_02hex(c + m)}`;
    } else if (h < 300) {
        return `#${to_02hex(x + m)}${to_02hex(m)}${to_02hex(c + m)}`;
    } else {
        return `#${to_02hex(c + m)}${to_02hex(m)}${to_02hex(x + m)}`;
    }
};

export class Colors {
    public static red = (i: number): string => {
        const colors = [
            hsl_to_rgb(0, 206, 110),
            hsl_to_rgb(340, 206, 110),
            hsl_to_rgb(20, 206, 110),
            hsl_to_rgb(0, 206, 150),
            hsl_to_rgb(340, 206, 150),
            hsl_to_rgb(20, 206, 150),
            hsl_to_rgb(0, 206, 90),
            hsl_to_rgb(340, 206, 90),
            hsl_to_rgb(20, 206, 90),
        ];

        return colors[i % colors.length];
    };

    public static yellow = (i: number): string => {
        const colors = [
            hsl_to_rgb(50, 230, 128),
            hsl_to_rgb(40, 230, 128),
            hsl_to_rgb(60, 230, 128),
            hsl_to_rgb(50, 230, 160),
            hsl_to_rgb(40, 230, 160),
            hsl_to_rgb(60, 230, 160),
            hsl_to_rgb(50, 230, 96),
            hsl_to_rgb(40, 230, 96),
            hsl_to_rgb(60, 230, 96),
        ];

        return colors[i % colors.length];
    };

    public static green = (i: number): string => {
        const colors = [
            hsl_to_rgb(115, 128, 90),
            hsl_to_rgb(105, 128, 90),
            hsl_to_rgb(125, 128, 90),
            hsl_to_rgb(115, 128, 120),
            hsl_to_rgb(105, 128, 120),
            hsl_to_rgb(125, 128, 120),
            hsl_to_rgb(115, 128, 60),
            hsl_to_rgb(105, 128, 60),
            hsl_to_rgb(125, 128, 60),
        ];

        return colors[i % colors.length];
    };

    public static blue = (i: number): string => {
        const colors = [
            hsl_to_rgb(220, 186, 110),
            hsl_to_rgb(210, 186, 110),
            hsl_to_rgb(230, 186, 110),
            hsl_to_rgb(220, 186, 150),
            hsl_to_rgb(210, 186, 150),
            hsl_to_rgb(230, 186, 150),
            hsl_to_rgb(220, 186, 90),
            hsl_to_rgb(210, 186, 90),
            hsl_to_rgb(230, 186, 90),
        ];

        return colors[i % colors.length];
    };

    public static purple = (i: number): string => {
        const colors = [
            hsl_to_rgb(280, 130, 120),
            hsl_to_rgb(270, 130, 120),
            hsl_to_rgb(290, 130, 120),
            hsl_to_rgb(280, 130, 150),
            hsl_to_rgb(270, 130, 150),
            hsl_to_rgb(290, 130, 150),
            hsl_to_rgb(280, 130, 90),
            hsl_to_rgb(270, 130, 90),
            hsl_to_rgb(290, 130, 90),
        ];

        return colors[i % colors.length];
    };
}
