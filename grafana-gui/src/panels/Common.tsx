import {
    SceneDataTransformer,
    SceneObject,
    SceneObjectState,
    SceneQueryRunner,
} from "@grafana/scenes";

export type Panel = {
    query: SceneQueryRunner | SceneDataTransformer;
    scene: SceneObject<SceneObjectState>;
};

const to_02hex = (x: number): string => {
    let hex = Math.round(x).toString(16);
    if (hex.length < 2) {
        hex = `0${hex}`;
    }
    return hex;
};

export const hsl_to_rgb = (h: number, s: number, l: number): string => {
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
