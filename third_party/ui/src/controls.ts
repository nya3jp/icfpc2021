import {Editor, ProblemSet} from './editor';

export class ProblemSelector {
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

export class OutputTextArea {
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

export class DistanceToggle {
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

export class ZoomSlider {
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

export class StatusLabel {
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
