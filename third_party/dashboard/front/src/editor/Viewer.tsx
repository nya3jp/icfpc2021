import {Point, ProblemData, SolutionData} from '../types';
import React, {useEffect, useRef} from 'react';

function distance(p: Point, q: Point) {
    const dx = p[0] - q[0];
    const dy = p[1] - q[1];
    return dx * dx + dy * dy;
}

class Translator {
    private center: Point = [0, 0];

    constructor(public zoom: number = 5.0) {}

    static fitTo(points: Point[], canvasWidth: number, canvasHeight: number): Translator {
        let maxX = 1.0, maxY = 1.0;
        for (const p of points) {
            maxX = Math.max(maxX, p[0]);
            maxY = Math.max(maxY, p[1]);
        }
        const zoom = Math.min(canvasWidth / (maxX + 10), canvasHeight / (maxY + 10));
        return new Translator(zoom);
    }

    modelToCanvas(p: Point): Point {
        return [(p[0] - this.center[0]) * this.zoom,
            (p[1] - this.center[1]) * this.zoom];
    }

    canvasToModel(p: Point): Point {
        return [p[0] / this.zoom + this.center[0],
            p[1] / this.zoom + this.center[1]];
    }

    moveCenter(p: Point) {
        this.center = [this.center[0] + p[0], this.center[1] + p[1]]
    }
}

function draw(canvas: HTMLCanvasElement, problem: ProblemData, solution?: SolutionData) {
    // TODO: Consider problem.hole and solution.vertices.
    const translator = Translator.fitTo(problem.figure.vertices, canvas.width, canvas.height);

    const ctx = canvas.getContext('2d')!;
    ctx.fillStyle = 'rgb(222, 222, 222)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw hole.
    if (problem.hole.length > 0) {
        ctx.fillStyle = 'rgb(255, 255, 255)';
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.beginPath();
        ctx.moveTo(...translator.modelToCanvas(problem.hole[problem.hole.length - 1]));
        for (const v of problem.hole) {
            ctx.lineTo(...translator.modelToCanvas(v));
        }
        ctx.fill();
        ctx.stroke();
    }

    // TODO: Consider drawing the original pose.

    // Draw pose.
    const {edges, vertices} = problem.figure;
    const pose = solution ? solution.vertices : vertices;

    ctx.strokeStyle = 'rgb(255, 0, 0)';
    for (const edge of edges) {
        ctx.strokeStyle = getLineColor(distance(pose[edge[0]], pose[edge[1]]), distance(vertices[edge[0]], vertices[edge[1]]), problem.epsilon);
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
    problem: ProblemData;
    solution?: SolutionData;
}

export function Viewer(props: ViewerProps) {
    const {problem, solution} = props;
    const canvasRef = useRef<HTMLCanvasElement | null>(null);

    useEffect(() => {
        draw(canvasRef.current!, problem, solution);
    }, []);

    return <canvas ref={canvasRef} width={400} height={400} style={{border: '3px solid black', userSelect: 'none'}} />;
}
