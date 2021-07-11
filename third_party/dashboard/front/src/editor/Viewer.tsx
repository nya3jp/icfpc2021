import {Point, Problem, Solution} from '../types';
import React, {useEffect, useRef} from 'react';
import {boundingBox, distance2, midPoint, vadd, vdiv, vmul, vsub} from './geom';

class Translator {
    constructor(public zoom: number, public offset: Point = [0, 0]) {}

    static fitTo(points: Point[], canvasWidth: number, canvasHeight: number): Translator {
        const [bbMin, bbMax] = boundingBox(points);
        const center = midPoint(bbMin, bbMax);
        const zoom = Math.min(canvasWidth / (bbMax[0] - bbMin[0]), canvasHeight / (bbMax[1] - bbMin[1])) * 0.95;
        const offset = vsub(vdiv([canvasWidth / 2, canvasHeight / 2], zoom), center);
        return new Translator(zoom, offset);
    }

    public modelToCanvas(p: Point): Point {
        return vmul(vadd(p, this.offset), this.zoom);
    }

    public canvasToModel(p: Point): Point {
        return vsub(vdiv(p, this.zoom), this.offset);
    }
}

function draw(canvas: HTMLCanvasElement, problem: Problem, solution?: Solution) {
    // TODO: Consider problem.hole and solution.vertices.
    const points = problem.data.hole.concat(solution ? solution.data.vertices : problem.data.figure.vertices);
    const translator = Translator.fitTo(points, canvas.width, canvas.height);

    const ctx = canvas.getContext('2d')!;
    ctx.fillStyle = 'rgb(222, 222, 222)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw hole.
    const hole = problem.data.hole;
    if (hole.length > 0) {
        ctx.fillStyle = 'rgb(255, 255, 255)';
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.beginPath();
        ctx.moveTo(...translator.modelToCanvas(hole[hole.length - 1]));
        for (const v of hole) {
            ctx.lineTo(...translator.modelToCanvas(v));
        }
        ctx.fill();
        ctx.stroke();
    }

    // Draw bonuses.
    const radius = 3.0 * translator.zoom;
    ctx.strokeStyle = 'rgba(255, 255, 0, 1)';
    ctx.fillStyle = 'rgba(255, 255, 0, 0.5)';
    for (const bonus of problem.data.bonuses) {
        ctx.beginPath();
        const pos = translator.modelToCanvas(bonus.position);
        ctx.arc(pos[0], pos[1], radius, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();
    }

    // Draw pose.
    const {edges, vertices} = problem.data.figure;
    const pose = solution ? solution.data.vertices : vertices;

    ctx.strokeStyle = 'rgb(255, 0, 0)';
    for (const edge of edges) {
        ctx.strokeStyle = getLineColor(distance2(pose[edge[0]], pose[edge[1]]), distance2(vertices[edge[0]], vertices[edge[1]]), problem.data.epsilon);
        ctx.beginPath();
        ctx.moveTo(...translator.modelToCanvas(pose[edge[0]]));
        ctx.lineTo(...translator.modelToCanvas(pose[edge[1]]));
        ctx.stroke();
    }
    ctx.fillStyle = 'rgb(0, 0, 255)';
    for (const vertex of pose) {
        const [x, y] = translator.modelToCanvas(vertex);
        ctx.beginPath();
        ctx.arc(x, y, 2.0, 0, 2*Math.PI);
        ctx.fill();
    }
}

function getLineColor(current: number, original: number, epsilon: number): string {
    const margin = original * epsilon / 1000000;
    const min = original - margin;
    const max = original + margin;
    if (current < min) {
        return 'rgb(255, 0, 0)';
    } else if (current > max) {
        return 'rgb(0, 0, 255)';
    }
    return 'rgb(0, 255, 0)'
}

interface ViewerProps {
    problem: Problem;
    solution?: Solution;
    size?: number;
}

export function Viewer(props: ViewerProps) {
    const {problem, solution, size = 400} = props;
    const canvasRef = useRef<HTMLCanvasElement | null>(null);

    useEffect(() => {
        draw(canvasRef.current!, problem, solution);
    }, []);

    return <canvas ref={canvasRef} width={size} height={size} style={{border: '3px solid black', userSelect: 'none'}} />;
}
