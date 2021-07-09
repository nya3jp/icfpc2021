import {Figure, Hole, Point, Problem} from './types';
import { fetch_problem, problem_list } from './problem_fetcher';

function distance(p: Point, q: Point) {
    const dx = p[0] - q[0];
    const dy = p[1] - q[1];
    return dx * dx + dy * dy;
}

function roundPoint(p: Point): Point {
    return [Math.round(p[0]), Math.round(p[1])];
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
        private readonly output: HTMLTextAreaElement,
        private problem: Problem,
        private readonly translator: Translator = new Translator()) {
        this.pose = [...problem.figure.vertices];
    }

    public start() {
        this.canvas.addEventListener('mousedown', (ev) => this.onMouseDown(ev));
        this.canvas.addEventListener('mouseup', (ev) => this.onMouseUp(ev));
        this.canvas.addEventListener('mousemove', (ev) => this.onMouseMove(ev));
        this.draw();
    }

    public resetProblem(newProblem: Problem) {
        this.problem = newProblem;
        this.pose = [...this.problem.figure.vertices];
        this.draw();
    }

    private draw() {
        const ctx = this.canvas.getContext('2d')!;
        ctx.fillStyle = 'rgb(222, 222, 222)';
        ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.drawHole(ctx);
        this.drawFigure(ctx);
        this.output.textContent = JSON.stringify({vertices: this.pose});
    }

    private drawHole(ctx: CanvasRenderingContext2D) {
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

    private drawFigure(ctx: CanvasRenderingContext2D) {
        const {edges, vertices} = this.problem.figure;
        const pose = this.pose;
        ctx.strokeStyle = 'rgb(255, 0, 0)';
        for (const edge of edges) {
            const ok = (distance(pose[edge[0]], pose[edge[1]]) / distance(vertices[edge[0]], vertices[edge[1]]) - 1) <= this.problem.epsilon / 1000000;
            ctx.strokeStyle = ok ? 'rgb(0, 255, 0)' : 'rgb(255, 0, 0)';
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

    private onDragVertex(pos: Point) {
        this.pose[this.draggingVertex!] = roundPoint(pos);
        this.draw();
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
        fetch_problem(parseInt(id)).then(problem => {
            this.ui.resetProblem(problem)
        })
    }
}

const theFirstProblem = {"hole":[[45,80],[35,95],[5,95],[35,50],[5,5],[35,5],[95,95],[65,95],[55,80]],"epsilon":150000,"figure":{"edges":[[2,5],[5,4],[4,1],[1,0],[0,8],[8,3],[3,7],[7,11],[11,13],[13,12],[12,18],[18,19],[19,14],[14,15],[15,17],[17,16],[16,10],[10,6],[6,2],[8,12],[7,9],[9,3],[8,9],[9,12],[13,9],[9,11],[4,8],[12,14],[5,10],[10,15]],"vertices":[[20,30],[20,40],[30,95],[40,15],[40,35],[40,65],[40,95],[45,5],[45,25],[50,15],[50,70],[55,5],[55,25],[60,15],[60,35],[60,65],[60,95],[70,95],[80,30],[80,40]]}} as Problem;

const ui = new UI(
    document.getElementById('canvas') as HTMLCanvasElement,
    document.getElementById('output') as HTMLTextAreaElement,
    theFirstProblem
);
ui.start()

let select: HTMLSelectElement = document.getElementById("problems") as HTMLSelectElement;
const dropdown = new ProblemDropDownMenu(
    select,
    ui
);