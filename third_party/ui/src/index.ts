import {Figure, Hole, Point, Problem} from './types';
import { fetch_problem } from './problem_fetcher';

class Translator {
    constructor(public zoom: number = 5.0) {}

    modelToCanvas(p: Point): Point {
        return [p[0] * this.zoom, p[1] * this.zoom];
    }

    canvasToModel(p: Point): Point {
        return [p[0] / this.zoom, p[1] / this.zoom];
    }
}

class UI {
    private draggingVertex?: number;

    constructor(
        private readonly canvas: HTMLCanvasElement,
        private readonly problem: Problem,
        private readonly translator: Translator = new Translator()) {
    }

    start() {
        this.canvas.addEventListener('mousedown', (ev) => this.onMouseDown(ev));
        this.canvas.addEventListener('mouseup', (ev) => this.onMouseUp(ev));
        this.draw();
    }

    draw() {
        const ctx = this.canvas.getContext('2d')!;
        this.drawHole(ctx, this.problem.hole);
        this.drawFigure(ctx, this.problem.figure);
    }

    drawHole(ctx: CanvasRenderingContext2D, hole: Hole) {
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.beginPath();
        ctx.moveTo(...this.translator.modelToCanvas(hole[hole.length - 1]));
        for (const v of this.problem.hole) {
            ctx.lineTo(...this.translator.modelToCanvas(v));
        }
        ctx.stroke();
    }

    drawFigure(ctx: CanvasRenderingContext2D, figure: Figure) {
        const {edges, vertices} = figure;
        ctx.strokeStyle = 'rgb(255, 0, 0)';
        for (const edge of edges) {
            ctx.beginPath();
            ctx.moveTo(...this.translator.modelToCanvas(vertices[edge[0]]));
            ctx.lineTo(...this.translator.modelToCanvas(vertices[edge[1]]));
            ctx.stroke();
        }
    }

    onMouseDown(ev: MouseEvent) {
        if (ev.button !== 0) {
            return;
        }
        const p = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
    }

    onMouseUp(ev: MouseEvent) {
        if (ev.button !== 0) {
            return;
        }
    }
}

fetch_problem(1).then(problem => {
    const ui = new UI(
        document.getElementById('canvas') as HTMLCanvasElement,
        problem
    )
    ui.start()
})
