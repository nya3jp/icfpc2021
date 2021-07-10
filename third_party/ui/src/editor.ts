import {Point, Pose, Problem} from './types';
import {
    distance2,
    midPoint,
    roundPoint,
    vabs,
    vadd,
    vdiv,
    vdot,
    vmul,
    vsub,
    vunit
} from './geom';
import {forceLayout} from './layout';

const deepEqual = require('deep-equal');

export class Translator {
    public offset: Point = [0, 0];

    constructor(public zoom: number) {
    }

    public modelToCanvas(p: Point): Point {
        return vmul(vadd(p, this.offset), this.zoom);
    }

    public canvasToModel(p: Point): Point {
        return vsub(vdiv(p, this.zoom), this.offset);
    }
}

interface Highlight {
    holeEdge?: number;
}

export class Editor extends EventTarget {
    private problem: Problem = {
        hole: [],
        figure: {edges: [], vertices: []},
        epsilon: 0
    };
    private pose: Pose = [];

    private currentHighlight: Highlight = {};
    private draggingVertex: number | null = null;
    private slideStartCenter: Point | null = null;
    private slideStartCanvas: Point | null = null;

    constructor(
        private readonly canvas: HTMLCanvasElement,
        private drawDistance: boolean = false,
        private similarEdgeHighlight: boolean = false,
        private constraintHint: boolean = false,
        private readonly translator: Translator = new Translator(5.0)) {
        super();
    }

    public start(): void {
        this.canvas.addEventListener('mousedown', (ev) => this.onMouseDown(ev));
        this.canvas.addEventListener('mouseup', (ev) => this.onMouseUp(ev));
        this.canvas.addEventListener('mousemove', (ev) => this.onMouseMove(ev));
        this.canvas.addEventListener('contextmenu', (ev) => {
            ev.preventDefault();
            return false;
        });
        this.canvas.addEventListener('wheel', (ev) => this.onMouseWheel(ev), {passive: false})
        this.render();
    }

    public getZoom(): number {
        return this.translator.zoom;
    }

    public setZoom(zoom: number) {
        this.translator.zoom = zoom;
        this.render();
    }

    public setProblem(problem: Problem): void {
        this.problem = problem;
        this.pose = [...problem.figure.vertices];
        this.render();
    }

    public setDrawDistance(drawDistance: boolean): void {
        this.drawDistance = drawDistance;
        this.render();
    }

    public setSimilarEdgeHighlight(similarEdgeHighlight: boolean): void {
        this.similarEdgeHighlight = similarEdgeHighlight;
        this.render();
    }

    public setConstraintHint(constraintHint: boolean): void {
        this.constraintHint = constraintHint;
        this.render();
    }

    public getPose(): Pose {
        return this.pose;
    }

    public setPose(pose: Pose): void {
        this.pose = pose;
        this.render();
    }

    public computeDislike(): number {
        let dislike = 0;
        for (const h of this.problem.hole) {
            dislike += this.pose
                .map((p) => distance2(p, h))
                .reduce((a, b) => Math.min(a, b));
        }
        return dislike;
    }

    private render(): void {
        const ctx = this.canvas.getContext('2d')!;
        ctx.fillStyle = 'rgb(222, 222, 222)';
        ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.renderHole(ctx);
        this.renderPose(ctx);
        this.renderDistance(ctx);
        this.renderHints(ctx);
        this.dispatchEvent(new CustomEvent('refresh'));
    }

    private renderHole(ctx: CanvasRenderingContext2D): void {
        const {hole} = this.problem;
        if (hole.length === 0) {
            return;
        }
        ctx.fillStyle = 'rgb(255, 255, 255)';
        ctx.strokeStyle = 'rgb(0, 0, 0)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(...this.translator.modelToCanvas(hole[hole.length - 1]));
        for (const v of hole) {
            ctx.lineTo(...this.translator.modelToCanvas(v));
        }
        ctx.fill();
        ctx.stroke();

        if (this.currentHighlight.holeEdge !== undefined) {
            const i = this.currentHighlight.holeEdge;
            ctx.strokeStyle = 'rgb(0, 0, 0)';
            ctx.lineWidth = 3;
            ctx.beginPath();
            ctx.moveTo(...this.translator.modelToCanvas(hole[i]));
            ctx.lineTo(...this.translator.modelToCanvas(hole[(i + 1) % hole.length]));
            ctx.stroke();
        }
    }

    private renderPose(ctx: CanvasRenderingContext2D): void {
        let target2 = -1;
        if (this.currentHighlight.holeEdge !== undefined) {
            const {hole} = this.problem;
            const i = this.currentHighlight.holeEdge;
            target2 = distance2(hole[i], hole[(i + 1) % hole.length]);
        }
        const {edges, vertices} = this.problem.figure;
        const pose = this.pose;
        for (const edge of edges) {
            let highlight = false;
            if (target2 >= 0) {
                const original2 = distance2(vertices[edge[0]], vertices[edge[1]]);
                if (Math.abs(target2 / original2 - 1) < this.problem.epsilon / 1000000) {
                    highlight = true;
                }
            }
            if (this.draggingVertex !== null && edge[0] === this.draggingVertex || edge[1] == this.draggingVertex) {
                highlight = true;
            }
            ctx.lineWidth = highlight ? 3 : 1;
            ctx.strokeStyle = this.getLineColor(distance2(pose[edge[0]], pose[edge[1]]), distance2(vertices[edge[0]], vertices[edge[1]]), highlight);
            ctx.beginPath();
            ctx.moveTo(...this.translator.modelToCanvas(pose[edge[0]]));
            ctx.lineTo(...this.translator.modelToCanvas(pose[edge[1]]));
            ctx.stroke();
        }
        ctx.fillStyle = 'rgb(0, 0, 255)';
        for (const vertex of this.pose) {
            const [x, y] = this.translator.modelToCanvas(vertex);
            ctx.beginPath();
            ctx.arc(x, y, 2.0, 0, 2 * Math.PI);
            ctx.fill();
        }
    }

    private getLineColor(current: number, original: number, highlight: boolean): string {
        const hi = highlight ? 192 : 255;
        const lo = 0;
        const margin = original * this.problem.epsilon / 1000000;
        const min = original - margin;
        const max = original + margin;
        if (current < min) {
            return `rgb(${hi}, ${lo}, ${lo})`;
        } else if (current > max) {
            return `rgb(${lo}, ${lo}, ${hi})`;
        }
        return `rgb(${lo}, ${hi}, ${lo})`
    }

    private renderDistance(ctx: CanvasRenderingContext2D): void {
        if (!this.drawDistance) {
            return;
        }

        ctx.font = "11px serif";
        ctx.strokeStyle = 'rgb(10, 10, 10)';
        ctx.lineWidth = 1;

        const {edges, vertices} = this.problem.figure;
        const pose = this.pose;
        for (const edge of edges) {
            const dist = distance2(pose[edge[0]], pose[edge[1]]);
            const original = distance2(vertices[edge[0]], vertices[edge[1]]);
            const margin = original * this.problem.epsilon / 1000000;
            const mid = this.translator.modelToCanvas(midPoint(pose[edge[0]], pose[edge[1]]));
            const text = dist.toString() + "∈ [" + Math.ceil(original - margin).toString()
                + "," + Math.floor(original + margin).toString() + "]";
            ctx.strokeText(text, mid[0], mid[1]);
            ctx.fillStyle = this.getLineColor(dist, original, false);
            ctx.fillText(text, mid[0], mid[1]);
        }
    }

    private renderHints(ctx: CanvasRenderingContext2D): void {
        if (this.constraintHint && this.draggingVertex !== null) {
            const {edges, vertices} = this.problem.figure;
            const adjacents = [];
            for (const edge of edges) {
                if (edge[0] === this.draggingVertex) {
                    adjacents.push(edge[1]);
                }
                if (edge[1] === this.draggingVertex) {
                    adjacents.push(edge[0]);
                }
            }
            ctx.fillStyle = 'rgba(0, 255, 0, 0.15)';
            for (const adjacent of adjacents) {
                const original2 = distance2(vertices[adjacent], vertices[this.draggingVertex]);
                const margin2 = original2 * this.problem.epsilon / 1000000;
                const min = Math.sqrt(original2 - margin2);
                const max = Math.sqrt(original2 + margin2);
                const center = this.pose[adjacent];
                const minCanvas = min * this.translator.zoom;
                const maxCanvas = max * this.translator.zoom;
                const centerCanvas = this.translator.modelToCanvas(center);
                ctx.beginPath();
                ctx.arc(centerCanvas[0], centerCanvas[1], maxCanvas, 0, Math.PI * 2, false);
                ctx.arc(centerCanvas[0], centerCanvas[1], minCanvas, 0, Math.PI * 2, true);
                ctx.fill();
            }
        }
    }

    private onMouseDown(ev: MouseEvent): void {
        switch (ev.button) {
            case 0: // Left click
                const pos = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
                let nearest = 0;
                for (let i = 0; i < this.pose.length; ++i) {
                    if (distance2(this.pose[i], pos) < distance2(this.pose[nearest], pos)) {
                        nearest = i;
                    }
                }
                if (distance2(this.pose[nearest], pos) < 10 * 10) {
                    this.draggingVertex = nearest;
                    this.onDragVertex(pos);
                }
                break;
            case 2: // Right click
                this.slideStartCanvas = [ev.offsetX, ev.offsetY];
                this.slideStartCenter = this.translator.offset;
                break;
        }
    }

    private onMouseUp(ev: MouseEvent): void {
        switch (ev.button) {
            case 0: // Left click
                this.draggingVertex = null;
                this.render();
                break;
            case 2: // Right click
                this.slideStartCanvas = null;
                this.slideStartCenter = null;
                this.render();
                break;
        }
    }

    private onMouseMove(ev: MouseEvent): void {
        const mouse: Point = [ev.offsetX, ev.offsetY];
        if (this.draggingVertex !== null) {
            const pos = this.translator.canvasToModel(mouse);
            this.onDragVertex(pos);
            this.render();
        }
        if (this.slideStartCanvas !== null) {
            const delta = vdiv(vsub(mouse, this.slideStartCanvas), this.translator.zoom);
            this.translator.offset = vadd(this.slideStartCenter!, delta);
            this.render();
        }
        const highlight: Highlight = {};
        if (this.draggingVertex === null && this.slideStartCanvas === null) {
            if (this.similarEdgeHighlight) {
                highlight.holeEdge = this.nearHoleEdge(this.translator.canvasToModel(mouse), 50 / this.translator.zoom);
            }
        }
        if (!deepEqual(highlight, this.currentHighlight)) {
            this.currentHighlight = highlight;
            this.render();
        }
    }

    private onMouseWheel(ev: WheelEvent): void {
        ev.preventDefault();
        const mouse: Point = [ev.offsetX, ev.offsetY];
        const oldZoom = this.translator.zoom;
        const newZoom = Math.min(20, Math.max(1, oldZoom + ev.deltaY / 200));
        const deltaZoom = newZoom - oldZoom;
        this.translator.offset = vsub(this.translator.offset, vmul(mouse, deltaZoom / (oldZoom * (oldZoom + deltaZoom))));
        this.translator.zoom = newZoom;
        this.render();
    }

    private onDragVertex(pos: Point): void {
        this.pose[this.draggingVertex!] = roundPoint(pos);
        this.render();
    }

    private nearHoleEdge(p: Point, threshold: number): number | undefined {
        const {hole} = this.problem;
        let bestIndex: number | undefined;
        let bestDist2 = 1e10;
        for (let i = 0; i < hole.length; i++) {
            const a = hole[i];
            const b = hole[(i + 1) % hole.length];
            const delta = vsub(b, a);
            const t = Math.max(0, Math.min(1, vdot(vsub(p, a), vunit(delta)) / vabs(delta)));
            const nearest = vadd(a, vmul(delta, t));
            const dist2 = distance2(p, nearest);
            if (dist2 < bestDist2) {
                bestIndex = i;
                bestDist2 = dist2;
            }
        }
        if (bestDist2 >= threshold) {
            return undefined;
        }
        return bestIndex;
    }

    public relayout(): void {
        const {edges, vertices} = this.problem.figure;
        this.pose = forceLayout(this.pose, edges, 1000);
        let scale = 0;
        for (const [a, b] of edges) {
            scale += vabs(vsub(vertices[a], vertices[b])) / vabs(vsub(this.pose[a], this.pose[b]));
        }
        scale /= edges.length;
        for (let i = 0; i < this.pose.length; ++i) {
            this.pose[i] = vmul(this.pose[i], scale);
        }
        let minX = 1e10, minY = 1e10;
        for (const p of this.pose) {
            minX = Math.min(minX, p[0]);
            minY = Math.min(minY, p[1]);
        }
        for (let i = 0; i < this.pose.length; ++i) {
            this.pose[i] = vsub(this.pose[i], [minX, minY]);
        }
        this.render();
    }
}
