import {Point} from './types';

export function distance(p: Point, q: Point): number {
    const dx = p[0] - q[0];
    const dy = p[1] - q[1];
    return dx * dx + dy * dy;
}

export function roundPoint(p: Point): Point {
    return [Math.round(p[0]), Math.round(p[1])];
}

export function midPoint(p: Point, q: Point): Point {
    return [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2];
}

export class Translator {
    public center: Point = [0, 0];

    constructor(public zoom: number) {
    }

    public modelToCanvas(p: Point): Point {
        return [(p[0] - this.center[0]) * this.zoom,
            (p[1] - this.center[1]) * this.zoom];
    }

    public canvasToModel(p: Point): Point {
        return [p[0] / this.zoom + this.center[0],
            p[1] / this.zoom + this.center[1]];
    }
}
