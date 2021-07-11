import {Editor} from './editor';
import {ProblemSet} from './problems';

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

export class FragmentUpdater {
    constructor(
        private readonly editor: Editor,
        private readonly problemSelector: ProblemSelector) {
    }

    public start(): void {
        this.editor.addEventListener('refresh', () => this.onRefresh());
        this.onRefresh();
    }

    private onRefresh(): void {
        const problemId = this.problemSelector.getProblemId()
        if (Number.isNaN(problemId)) {
            // The UI is initializing. Do not update the fragment.
            return;
        }
        const pose = this.editor.getPose();
        const url = new URL('/', 'http://example.com');
        url.searchParams.set('problem_id', problemId.toString());
        url.searchParams.set('pose', JSON.stringify(pose));
        window.location.hash = '#?' + url.searchParams.toString();
    }
}

export class SimilarEdgeHighlightToggle {
    constructor(
        private readonly checkbox: HTMLInputElement,
        private readonly editor: Editor) {
    }

    public start(): void {
        this.checkbox.addEventListener('change', () => this.onChange());
        this.onChange();
    }

    private onChange(): void {
        this.editor.setSimilarEdgeHighlight(this.checkbox.checked);
    }
}

export class ConstraintHintToggle {
    constructor(
        private readonly checkbox: HTMLInputElement,
        private readonly editor: Editor) {
    }

    public start(): void {
        this.checkbox.addEventListener('change', () => this.onChange());
        this.onChange();
    }

    private onChange(): void {
        this.editor.setConstraintHint(this.checkbox.checked);
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

export class RelayoutButton {
    constructor(
        private readonly element: HTMLButtonElement,
        private readonly editor: Editor) {
    }

    public start(): void {
        this.element.addEventListener('click', () => this.onClick());
    }

    private onClick(): void {
        if (!window.confirm('Do you really want to run relayout?')) {
            return;
        }
        this.editor.relayout();
    }
}
