import {Figure, Hole, Problem} from './types';
import { fetch_problem } from './problem_fetcher';

const theProblem: Problem = fetch_problem(0);

const theCanvas = document.getElementById('canvas') as HTMLCanvasElement;

function drawHole(ctx: CanvasRenderingContext2D, hole: Hole) {
    ctx.strokeStyle = 'rgb(0, 0, 0)';
    ctx.beginPath();
    ctx.moveTo(hole[hole.length - 1][0], hole[hole.length - 1][1]);
    for (const v of theProblem.hole) {
        ctx.lineTo(v[0], v[1]);
    }
    ctx.stroke();
}

function drawFigure(ctx: CanvasRenderingContext2D, figure: Figure) {
    const {edges, vertices} = figure;
    ctx.strokeStyle = 'rgb(255, 0, 0)';
    for (const edge of edges) {
        ctx.beginPath();
        const p = vertices[edge[0]];
        const q = vertices[edge[1]];
        ctx.moveTo(p[0], p[1]);
        ctx.lineTo(q[0], q[1]);
        ctx.stroke();
    }
}

function draw() {
    const ctx = theCanvas.getContext('2d')!;
    drawHole(ctx, theProblem.hole);
    drawFigure(ctx, theProblem.figure);
}

draw();
