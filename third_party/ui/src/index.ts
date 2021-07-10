import {Point, Pose, Problem} from './types';

function distance(p: Point, q: Point): number {
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
    public center: Point = [0, 0];

    constructor(public zoom: number = 5.0) {}

    public modelToCanvas(p: Point): Point {
        return [(p[0] - this.center[0]) * this.zoom,
            (p[1] - this.center[1]) * this.zoom];
    }

    public canvasToModel(p: Point): Point {
        return [p[0] / this.zoom + this.center[0],
            p[1] / this.zoom + this.center[1]];
    }
}

class Editor extends EventTarget {
    private problem: Problem = {hole: [], figure: {edges: [], vertices: []}, epsilon: 0};
    private pose: Pose = [];

    private draggingVertex: number | null = null;
    private slideStartCenter: Point | null = null;
    private slideStartCanvas: Point | null = null;

    constructor(
        private readonly canvas: HTMLCanvasElement,
        private drawDistance: boolean = false,
        private readonly translator: Translator = new Translator()) {
        super();
    }

    public start(): void {
        this.canvas.addEventListener('mousedown', (ev) => this.onMouseDown(ev));
        this.canvas.addEventListener('mouseup', (ev) => this.onMouseUp(ev));
        this.canvas.addEventListener('mousemove', (ev) => this.onMouseMove(ev));
        this.canvas.addEventListener('contextmenu', (ev) => { ev.preventDefault(); return false; });
        this.canvas.addEventListener('wheel', (ev) => this.onMouseWheel(ev), {passive: false})
        this.refresh();
    }

    public getZoom(): number {
        return this.translator.zoom;
    }

    public setZoom(zoom: number) {
        this.translator.zoom = zoom;
        this.refresh();
    }

    public setProblem(problem: Problem): void {
        this.problem = problem;
        this.pose = [...problem.figure.vertices];
        this.refresh();
    }

    public setDrawDistance(drawDistance: boolean): void {
        this.drawDistance = drawDistance;
        this.refresh();
    }

    public getPose(): Pose {
        return this.pose;
    }

    public setPose(pose: Pose): void {
        this.pose = pose;
        this.refresh();
    }

    public computeDislike(): number {
        let dislike = 0;
        for (const h of this.problem.hole) {
            dislike += this.pose
                .map((p) => distance(p, h))
                .reduce((a, b) => Math.min(a, b));
        }
        return dislike;
    }

    private refresh(): void {
        const ctx = this.canvas.getContext('2d')!;
        ctx.fillStyle = 'rgb(222, 222, 222)';
        ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.renderHole(ctx);
        this.renderPose(ctx);
        this.renderDistance(ctx);
        this.dispatchEvent(new CustomEvent('refresh'));
    }

    private renderHole(ctx: CanvasRenderingContext2D): void {
        const {hole} = this.problem;
        if (hole.length === 0) {
            return;
        }
        ctx.fillStyle = 'rgb(255, 255, 255)';
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.beginPath();
        ctx.moveTo(...this.translator.modelToCanvas(hole[hole.length - 1]));
        for (const v of hole) {
            ctx.lineTo(...this.translator.modelToCanvas(v));
        }
        ctx.fill();
        ctx.stroke();
    }

    private renderPose(ctx: CanvasRenderingContext2D): void {
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

    private renderDistance(ctx: CanvasRenderingContext2D): void {
        if (!this.drawDistance) {
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

    private onMouseDown(ev: MouseEvent): void {
        switch (ev.button) {
            case 0: // Left click
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
                break;
            case 2: // Right click
                this.slideStartCanvas = [ev.offsetX, ev.offsetY];
                this.slideStartCenter = this.translator.center;
                break;
        }
    }

    private onMouseUp(ev: MouseEvent): void {
        switch (ev.button) {
            case 0: // Left click
                this.draggingVertex = null;
                break;
            case 2: // Right click
                this.slideStartCanvas = null;
                this.slideStartCenter = null;
                break;
        }
    }

    private onMouseMove(ev: MouseEvent): void {
        if (this.draggingVertex !== null) {
            const pos = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
            this.onDragVertex(pos);
            this.refresh();
        }
        if (this.slideStartCanvas !== null) {
            const dx = (ev.offsetX - this.slideStartCanvas[0]) / this.translator.zoom;
            const dy = (ev.offsetY - this.slideStartCanvas[1]) / this.translator.zoom;
            this.translator.center = [this.slideStartCenter![0] - dx, this.slideStartCenter![1] - dy];
            this.refresh();
        }
    }

    private onMouseWheel(ev: WheelEvent): void {
        ev.preventDefault();
        this.translator.zoom = Math.min(20, Math.max(1, this.translator.zoom + ev.deltaY / 200));
        this.refresh();
    }

    private onDragVertex(pos: Point): void {
        this.pose[this.draggingVertex!] = roundPoint(pos);
        this.refresh();
    }
}

class ProblemSet {
    constructor() {}

    public async getProblemIds(): Promise<number[]> {
        return Array.from(Array(88).keys()).map(num => num + 1);
    }

    public async getProblem(id: number): Promise<Problem> {
        const res = await fetch(`./problems/${id}.problem`);
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json();
    }
}

class ProblemSelector {
    constructor(
        private readonly select: HTMLSelectElement,
        private readonly editor: Editor,
        private readonly problemSet: ProblemSet) {
    }

    public start(): void {
        this.select.addEventListener('change', () => this.onChange());
        this.insertItems();
    }

    public getProblemId(): number {
        return parseInt(this.select.value);
    }

    public async setProblemId(id: number): Promise<void> {
        const problem = await this.problemSet.getProblem(id);
        this.select.value = id.toString();
        this.editor.setProblem(problem);
    }

    private async insertItems(): Promise<void> {
        for (const id of await this.problemSet.getProblemIds()) {
            const option = new Option(id.toString(), id.toString());
            this.select.options.add(option);
        }
    }

    private onChange(): void {
        this.setProblemId(this.getProblemId());
    }
}

class OutputTextArea {
    constructor(
        private readonly textarea: HTMLTextAreaElement,
        private readonly editor: Editor,
        private readonly problemSelector: ProblemSelector) {
    }

    public start(): void {
        this.textarea.addEventListener('change', () => this.onChange());
        this.editor.addEventListener('refresh', () => this.onRefresh());
        this.onRefresh();
    }

    private onChange(): void {
        (async () => {
            const data = JSON.parse(this.textarea.value);
            const problemId = data['problem_id'];
            if (problemId !== undefined && problemId !== this.problemSelector.getProblemId()) {
                await this.problemSelector.setProblemId(problemId);
            }
            const pose = data['vertices'];
            if (pose.length !== this.editor.getPose().length) {
                return;
            }
            this.editor.setPose(pose);
        })();
    }

    private onRefresh(): void {
        this.textarea.value = JSON.stringify({
            problem_id: this.problemSelector.getProblemId(),
            vertices: this.editor.getPose(),
        });
    }
}

class DistanceToggle {
    constructor(
        private readonly checkbox: HTMLInputElement,
        private readonly editor: Editor) {
    }

    public start(): void {
        this.checkbox.addEventListener('change', () => this.onChange());
        this.onChange();
    }

    private onChange(): void {
        this.editor.setDrawDistance(this.checkbox.checked);
    }
}

class ZoomSlider {
    constructor(
        private readonly slider: HTMLInputElement,
        private readonly editor: Editor) {
    }

    public start(): void {
        this.slider.addEventListener('input', () => this.onInput());
        this.editor.addEventListener('refresh', () => this.onRefresh());
        this.onRefresh();
    }

    private onInput(): void {
        this.editor.setZoom(parseFloat(this.slider.value));
    }

    private onRefresh(): void {
        this.slider.value = this.editor.getZoom().toString();
    }
}

class StatusLabel {
    constructor(
        private readonly element: HTMLElement,
        private readonly editor: Editor) {
    }

    public start(): void {
        this.editor.addEventListener('refresh', () => this.onRefresh());
        this.onRefresh();
    }

    private onRefresh(): void {
        this.element.textContent = this.editor.computeDislike().toString();
    }
}

async function init() {
    const problemSet = new ProblemSet();
    const editor = new Editor(document.getElementById('canvas') as HTMLCanvasElement);
    const problemSelector = new ProblemSelector(document.getElementById("problems") as HTMLSelectElement, editor, problemSet);
    const outputTextArea = new OutputTextArea(document.getElementById('output') as HTMLTextAreaElement, editor, problemSelector);
    const zoomSlider = new ZoomSlider(document.getElementById('zoom') as HTMLInputElement, editor);
    const distanceToggle = new DistanceToggle(document.getElementById('show_distance') as HTMLInputElement, editor);
    const statusLabel = new StatusLabel(document.getElementById('dislike')!, editor);

    editor.start();
    problemSelector.start();
    outputTextArea.start();
    zoomSlider.start();
    distanceToggle.start();
    statusLabel.start();

    await problemSelector.setProblemId(1);
}

init();
