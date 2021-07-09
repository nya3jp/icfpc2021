import {Figure, Hole, Point, Problem} from './types';
import { fetch_problem } from './problem_fetcher';

function distance(p: Point, q: Point) {
    const dx = p[0] - q[0];
    const dy = p[1] - q[1];
    return dx * dx + dy * dy;
}

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
    private pose: Point[];
    private draggingVertex: number | null = null;

    constructor(
        private readonly canvas: HTMLCanvasElement,
        private readonly problem: Problem,
        private readonly translator: Translator = new Translator()) {
        this.pose = [...problem.figure.vertices];
    }

    public start() {
        this.canvas.addEventListener('mousedown', (ev) => this.onMouseDown(ev));
        this.canvas.addEventListener('mouseup', (ev) => this.onMouseUp(ev));
        this.canvas.addEventListener('mousemove', (ev) => this.onMouseMove(ev));
        this.draw();
    }

    private draw() {
        const ctx = this.canvas.getContext('2d')!;
        this.drawHole(ctx, this.problem.hole);
        this.drawFigure(ctx, this.problem.figure);
    }

    private drawHole(ctx: CanvasRenderingContext2D, hole: Hole) {
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.beginPath();
        ctx.moveTo(...this.translator.modelToCanvas(hole[hole.length - 1]));
        for (const v of this.problem.hole) {
            ctx.lineTo(...this.translator.modelToCanvas(v));
        }
        ctx.stroke();
    }

    private drawFigure(ctx: CanvasRenderingContext2D, figure: Figure) {
        const {edges, vertices} = figure;
        ctx.strokeStyle = 'rgb(255, 0, 0)';
        for (const edge of edges) {
            ctx.beginPath();
            ctx.moveTo(...this.translator.modelToCanvas(vertices[edge[0]]));
            ctx.lineTo(...this.translator.modelToCanvas(vertices[edge[1]]));
            ctx.stroke();
        }
    }

    private onMouseDown(ev: MouseEvent) {
        if (ev.button !== 0) {
            return;
        }
        const click = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
        let nearest = 0;
        for (let i = 0; i < this.pose.length; ++i) {
            if (distance(this.pose[i], click) < distance(this.pose[nearest], click)) {
                nearest = i;
            }
        }
        if (distance(this.pose[nearest], click) < 10*10) {
            this.draggingVertex = nearest;
            this.onDragVertex(click);
        }
    }

    private onMouseUp(ev: MouseEvent) {
        if (ev.button !== 0) {
            return;
        }
        this.draggingVertex = null;
    }

    private onMouseMove(ev: MouseEvent) {
        if (ev.button !== 0) {
            return;
        }
        if (this.draggingVertex === null) {
            return;
        }
        const click = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
        this.onDragVertex(click);
    }

    private onDragVertex(click: Point) {
        console.log(this.draggingVertex, click);
        this.draw();
    }
}

fetch_problem(1).then(problem => {
    const ui = new UI(
        document.getElementById('canvas') as HTMLCanvasElement,
        problem
    )
    ui.start()
})
