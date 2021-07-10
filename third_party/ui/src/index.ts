import {Figure, Hole, Point, Problem} from './types';
import { fetch_problem, problem_list } from './problem_fetcher';
import { createContext } from 'vm';

function distance(p: Point, q: Point) {
    const dx = p[0] - q[0];
    const dy = p[1] - q[1];
    return dx * dx + dy * dy;
}

function roundPoint(p: Point): Point {
    return [Math.round(p[0]), Math.round(p[1])];
}

function midPoint(p: Point, q: Point): Point {
    return [(p[0] + q[0]) / 2, (p[1] + q[1]) / 2];
}

class Translator {
    private center: Point = [0, 0];

    constructor(public zoom: number = 5.0) {}

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

class UI {
    private problem: Problem = {hole: [], figure: {edges: [], vertices: []}, epsilon: 0};
    private pose: Point[] = [];

    private draggingVertex: number | null = null;

    constructor(
        private readonly canvas: HTMLCanvasElement,
        private readonly output: HTMLTextAreaElement,
        private readonly zoom: HTMLInputElement,
        private problemId: number = 0,
        private readonly translator: Translator = new Translator()) {
    }

    public start() {
        this.canvas.addEventListener('mousedown', (ev) => this.onMouseDown(ev));
        this.canvas.addEventListener('mouseup', (ev) => this.onMouseUp(ev));
        this.canvas.addEventListener('mousemove', (ev) => this.onMouseMove(ev));
        document.addEventListener('keydown', (ev) => this.onKeyDown(ev))
        this.zoom.addEventListener('input', (ev) => this.onZoomChanged(ev))
        this.output.addEventListener('change', (ev) => this.onOutputChanged(ev));

        const checkbox = document.getElementById("show_distance") as HTMLInputElement;
        checkbox.addEventListener('change', (ev) => this.draw());

        this.draw();
    }

    public async loadProblem(id: number) {
        const problem = await fetch_problem(id);
        this.problemId = id;
        this.problem = problem;
        this.pose = [...problem.figure.vertices];
        this.draw();
    }

    private draw() {
        const ctx = this.canvas.getContext('2d')!;
        ctx.fillStyle = 'rgb(222, 222, 222)';
        ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.drawHole(ctx);
        this.drawPose(ctx);
        this.output.value = JSON.stringify({problem_id: this.problemId, vertices: this.pose});
        this.updateDislike();
        this.drawDistances(ctx);
    }

    private drawHole(ctx: CanvasRenderingContext2D) {
        if (this.problem.hole.length === 0) {
            return;
        }
        ctx.fillStyle = 'rgb(255, 255, 255)';
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.beginPath();
        ctx.moveTo(...this.translator.modelToCanvas(this.problem.hole[this.problem.hole.length - 1]));
        for (const v of this.problem.hole) {
            ctx.lineTo(...this.translator.modelToCanvas(v));
        }
        ctx.fill();
        ctx.stroke();
    }

    private drawPose(ctx: CanvasRenderingContext2D) {
        const {edges, vertices} = this.problem.figure;
        const pose = this.pose;
        ctx.strokeStyle = 'rgb(255, 0, 0)';
        for (const edge of edges) {
            ctx.strokeStyle = this.getLineColor(distance(pose[edge[0]], pose[edge[1]]), distance(vertices[edge[0]], vertices[edge[1]]));
            ctx.beginPath();
            ctx.moveTo(...this.translator.modelToCanvas(pose[edge[0]]));
            ctx.lineTo(...this.translator.modelToCanvas(pose[edge[1]]));
            ctx.stroke();
        }
        ctx.fillStyle = 'rgb(0, 0, 255)';
        for (const vertex of this.pose) {
            const [x, y] = this.translator.modelToCanvas(vertex);
            ctx.beginPath();
            ctx.arc(x, y, 2.0, 0, 2*Math.PI);
            ctx.fill();
        }
    }

    private getLineColor(current: number, original: number): string {
        const margin = original * this.problem.epsilon / 1000000;
        const min = original - margin;
        const max = original + margin;
        if (current < min) {
            return 'rgb(255, 0, 0)';
        } else if (current > max) {
            return 'rgb(0, 0, 255)';
        }
        return 'rgb(0, 255, 0)'
    }

    private drawDistances(ctx: CanvasRenderingContext2D) {
        const checkbox = document.getElementById("show_distance") as HTMLInputElement;
        if (!checkbox.checked) {
            return;
        }

        const {edges, vertices} = this.problem.figure;
        const pose = this.pose;
        for (const edge of edges) {
            const dist = distance(pose[edge[0]], pose[edge[1]]);
            const original = distance(vertices[edge[0]], vertices[edge[1]]);
            const margin = original * this.problem.epsilon / 1000000;
            const mid = this.translator.modelToCanvas(midPoint(pose[edge[0]], pose[edge[1]]));
            const text = dist.toString() + "∈ [" + Math.ceil(original - margin).toString()
                + "," + Math.floor(original + margin).toString() + "]";
            ctx.font = "11px serif";
            ctx.strokeStyle = 'rgb(10, 10, 10)';
            ctx.strokeText(text, mid[0], mid[1]);
            ctx.fillStyle = this.getLineColor(dist, original);
            ctx.fillText(text, mid[0], mid[1]);
        }
    }

    private onMouseDown(ev: MouseEvent) {
        if (ev.button !== 0) {
            return;
        }
        const pos = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
        let nearest = 0;
        for (let i = 0; i < this.pose.length; ++i) {
            if (distance(this.pose[i], pos) < distance(this.pose[nearest], pos)) {
                nearest = i;
            }
        }
        if (distance(this.pose[nearest], pos) < 10*10) {
            this.draggingVertex = nearest;
            this.onDragVertex(pos);
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
        const pos = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
        this.onDragVertex(pos);
    }

    private onKeyDown(ev: KeyboardEvent) {
        if (ev.code == "ArrowUp") {
            this.translator.moveCenter([0, -1])
            this.draw()
            ev.preventDefault()
        } else if (ev.code == "ArrowDown") {
            this.translator.moveCenter([0, 1])
            this.draw()
            ev.preventDefault()
        } else if (ev.code == "ArrowLeft") {
            this.translator.moveCenter([-1, 0])
            this.draw()
            ev.preventDefault()
        } else if (ev.code == "ArrowRight") {
            this.translator.moveCenter([1, 0])
            this.draw()
            ev.preventDefault()
        }
    }

    private onDragVertex(pos: Point) {
        this.pose[this.draggingVertex!] = roundPoint(pos);
        this.draw();
    }

    private onZoomChanged(ev: Event) {
        this.translator.zoom = parseFloat(this.zoom.value);
        this.draw();
    }

    private async onOutputChanged(ev: Event) {
        const parsed = JSON.parse(this.output.value);
        const problemId = parsed['problem_id'];
        if (problemId !== undefined && this.problemId !== problemId) {
            await this.loadProblem(problemId);
        }
        const pose = parsed['vertices'];
        if (pose.length === this.problem.figure.vertices.length) {
            this.pose = pose;
        }
        this.draw();
    }

    private updateDislike() {
        const dislike: HTMLDivElement = document.getElementById("dislike") as HTMLDivElement;
        dislike.textContent = this.calculateDislike().toString();
    }

    private calculateDislike() {
        let dislike = 0;
        for (var h of this.problem.hole) {
            dislike += this.pose
                .map(p => distance(p, h))
                .reduce((p, c) => Math.min(p, c))
        }
        return dislike;
    }
}

class ProblemDropDownMenu {
    menu: HTMLSelectElement;
    ui: UI;

    constructor(dropdown: HTMLSelectElement, ui: UI) {
        this.menu = dropdown;
        this.ui = ui;

        problem_list().forEach((element: number) => {
            const option = new Option(element.toString(), element.toString());
            this.menu.options.add(option);
        });

        dropdown.onchange = () => {
            this.OnChange();
        }
    }

    OnChange() {
        const id = this.menu.options[this.menu.selectedIndex].value;
        this.ui.loadProblem(parseInt(id));
    }
}

const ui = new UI(
    document.getElementById('canvas') as HTMLCanvasElement,
    document.getElementById('output') as HTMLTextAreaElement,
    document.getElementById('zoom') as HTMLInputElement,
);
ui.start();
ui.loadProblem(1);

let select: HTMLSelectElement = document.getElementById("problems") as HTMLSelectElement;
const dropdown = new ProblemDropDownMenu(
    select,
    ui
);