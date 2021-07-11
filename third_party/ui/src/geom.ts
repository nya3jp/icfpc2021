import {Point} from './types';

export function distance2(p: Point, q: Point): number {
    return vnorm(vsub(p, q));
}

export function roundPoint(p: Point): Point {
    return [Math.round(p[0]), Math.round(p[1])];
}

export function midPoint(p: Point, q: Point): Point {
    return vdiv(vadd(p, q), 2);
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

export function vnorm(p: Point): number {
    return p[0] * p[0] + p[1] * p[1];
}

export function vabs(p: Point): number {
    return Math.sqrt(vnorm(p));
}

export function vunit(p: Point): Point {
    return vdiv(p, vabs(p));
}

export function vdot(p: Point, q: Point): number {
    return p[0] * q[0] + p[1] * q[1];
}

export function closest(points: Point[], origin: Point): [Point, number] {
    return points
        .map((p, i) => [vabs(vsub(origin, p)), p, i])
        .reduce((a, b) => (a[0] < b[0] ? a : b)).slice(1) as [Point, number];
}
