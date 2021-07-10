import {Point} from './types';

export function distance2(p: Point, q: Point): number {
    const dx = p[0] - q[0];
    const dy = p[1] - q[1];
    return dx * dx + dy * dy;
}

export function roundPoint(p: Point): Point {
    return [Math.round(p[0]), Math.round(p[1])];
}

export function midPoint(p: Point, q: Point): Point {
    return vdiv(vadd(p, q), 2);
}

export class Translator {
    public center: Point = [0, 0];

    constructor(public zoom: number) {
    }

    public modelToCanvas(p: Point): Point {
        return vmul(vsub(p, this.center), this.zoom);
    }

    public canvasToModel(p: Point): Point {
        return vadd(vdiv(p, this.zoom), this.center);
    }
}

export function vadd(p: Point, q: Point): Point {
    return [p[0] + q[0], p[1] + q[1]];
}

export function vsub(p: Point, q: Point): Point {
    return [p[0] - q[0], p[1] - q[1]];
}

export function vmul(p: Point, m: number): Point {
    return [p[0] * m, p[1] * m];
}

export function vdiv(p: Point, m: number): Point {
    return [p[0] / m, p[1] / m];
}

export function vdot(p: Point, q: Point): number {
    return p[0] * q[0] + p[1] * q[1];
}
