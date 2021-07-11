import {Point, Pose, Problem} from './types';
import {
    boundingBox,
    closest,
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

interface Globalist {
    current: number;
    limit: number;
}

interface Highlight {
    holeEdge?: number;
}

export class Editor extends EventTarget {
    private problem: Problem = {
        hole: [],
        figure: {edges: [], vertices: []},
        epsilon: 0,
        bonuses: [],
    };
    private pose: Pose = [];

    private currentHighlight: Highlight = {};
    private draggingVertex: number | null = null;
    private slideStartCenter: Point | null = null;
    private slideStartCanvas: Point | null = null;

    constructor(
        private readonly canvas: HTMLCanvasElement,
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
        this.setZoomAt(zoom, [this.canvas.width / 2, this.canvas.height / 2]);
    }

    public setZoomAt(newZoom: number, centerCanvas: Point): void {
        newZoom = Math.min(20, Math.max(0.5, newZoom));
        const oldZoom = this.translator.zoom;
        this.translator.offset = vsub(this.translator.offset, vmul(centerCanvas, (newZoom - oldZoom) / (oldZoom * newZoom)));
        this.translator.zoom = newZoom;
        this.render();
    }

    public setZoomAutoFit(): void {
        const [bbMin, bbMax] = boundingBox(this.problem.hole.concat(this.pose));
        const center = midPoint(bbMin, bbMax);
        const zoom = Math.min(this.canvas.width / (bbMax[0] - bbMin[0]), this.canvas.height / (bbMax[1] - bbMin[1])) * 0.95;
        const offset = vsub(vdiv([this.canvas.width / 2, this.canvas.height / 2], zoom), center);
        this.translator.zoom = zoom;
        this.translator.offset = offset;
        this.render();
    }

    public setProblem(problem: Problem): void {
        this.problem = problem;
        this.pose = [...problem.figure.vertices];
        this.setZoomAutoFit();
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

    public computeGlobalist(): Globalist {
        const {edges, vertices} = this.problem.figure;
        const pose = this.pose;
        const globalist: Globalist = {
            current: 0,
            limit: this.problem.epsilon * edges.length / 1000000,
        };
        for (const edge of edges) {
            const original2 = distance2(vertices[edge[0]], vertices[edge[1]]);
            const dist2 = distance2(pose[edge[0]], pose[edge[1]]);
            globalist.current += Math.abs(dist2 / original2 - 1);
        }
        return globalist;
    }

    private render(): void {
        const ctx = this.canvas.getContext('2d')!;
        ctx.fillStyle = 'rgb(222, 222, 222)';
        ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.renderHole(ctx);
        this.renderBonuses(ctx);
        this.renderPose(ctx);
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

    private bonusColor(bonusType: string, alpha: number): string {
	    switch (bonusType) {
		    case 'GLOBALIST':
			    return `rgba(255, 255, 0, ${alpha})`;
		    case 'BREAK_A_LEG':
			    return `rgba(0, 0, 255, ${alpha})`;
		    case 'WALLHACK':
			    return `rgba(128, 128, 0, ${alpha})`;
		    default:
			    return `rgba(0, 0, 0, ${alpha})`;
	    }
    }

    private renderBonuses(ctx: CanvasRenderingContext2D): void {
        const radius = 3.0 * this.translator.zoom;
        for (const bonus of this.problem.bonuses) {
            ctx.strokeStyle = this.bonusColor(bonus.bonus, 1);
            ctx.fillStyle = this.bonusColor(bonus.bonus, 0.5);
            ctx.beginPath();
            const pos = this.translator.modelToCanvas(bonus.position);
            ctx.arc(pos[0], pos[1], radius, 0, Math.PI * 2);
            ctx.fill();
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
                if (Math.abs(target2 / original2 - 1) <= this.problem.epsilon / 1000000 + 1e-8) {
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
        const margin = original * this.problem.epsilon / 1000000 + 1e-8;
        const min = original - margin;
        const max = original + margin;
        if (current < min) {
            return `rgb(${hi}, ${lo}, ${lo})`;
        } else if (current > max) {
            return `rgb(${lo}, ${lo}, ${hi})`;
        }
        return `rgb(${lo}, ${hi}, ${lo})`
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
                const margin2 = original2 * this.problem.epsilon / 1000000 + 1e-8;
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
                    this.onDragVertex(ev);
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
            this.onDragVertex(ev);
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
        this.setZoomAt(this.translator.zoom + ev.deltaY / 200, [ev.offsetX, ev.offsetY]);
    }

    private onDragVertex(ev: MouseEvent): void {
        const cursor = this.translator.canvasToModel([ev.offsetX, ev.offsetY]);
        this.pose[this.draggingVertex!] = roundPoint(ev.shiftKey ? this.snap(cursor) : cursor);
        this.render();
    }

    private snap(cursor: Point): Point {
        const candidates = this.problem.bonuses.map(({position}) => position).concat(this.problem.hole);
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
        for (let dy = -5; dy <= 5; dy++) {
            for (let dx = -5; dx <= 5; dx++) {
                const d: Point = [dx, dy];
                if (vabs(d) > 5) {
                    continue;
                }
                const p = roundPoint(vadd(cursor, d));
                let ok = true;
                for (const adjacent of adjacents) {
                    const original2 = distance2(vertices[adjacent], vertices[this.draggingVertex!]);
                    const d2 = distance2(this.pose[adjacent], p);
                    const margin2 = original2 * this.problem.epsilon / 1000000 + 1e-8;
                    if (d2 < original2 - margin2 || d2 > original2 + margin2) {
                        ok = false;
                    }
                }
                if (ok) {
                    candidates.push(p);
                }
            }
        }
        const nearest = closest(candidates, cursor)[0];
        if (vabs(vsub(cursor, nearest)) < 30 / this.translator.zoom) {
            return nearest;
        }
        return cursor;
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
            this.pose[i] = roundPoint(vsub(this.pose[i], [minX, minY]));
        }
        this.render();
    }
}
