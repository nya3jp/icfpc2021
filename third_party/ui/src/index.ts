import {Editor} from './editor';
import {
    ConstraintHintToggle,
    DashboardSaver,
    FragmentUpdater,
    OutputTextArea,
    ProblemSelector, RelayoutButton, SimilarEdgeHighlightToggle,
    StatusLabel,
    ZoomSlider
} from './controls';
import {ProblemSet} from './problems';
import {SolutionSet} from './solutions';
import {Solution} from './types';

async function init() {
    const problemSet = new ProblemSet();
    const solutionSet = new SolutionSet();

    const editor = new Editor(document.getElementById('canvas') as HTMLCanvasElement);

    const problemSelector = new ProblemSelector(document.getElementById("problems") as HTMLSelectElement, editor, problemSet);
    const dashboardSaver = new DashboardSaver(
        document.getElementById("save_dashboard") as HTMLButtonElement,
        document.getElementById("save_dashboard_uiurl") as HTMLInputElement,
        document.getElementById("save_dashboard_dashurl") as HTMLInputElement,
        document.getElementById("save_dashboard_message") as HTMLSpanElement,
        solutionSet, editor, problemSelector);
    const fragmentUpdater = new FragmentUpdater(editor, problemSelector);
    const outputTextArea = new OutputTextArea(document.getElementById('output') as HTMLTextAreaElement, editor, problemSelector);
    const zoomSlider = new ZoomSlider(document.getElementById('zoom') as HTMLInputElement, editor);
    const similarEdgeHighlightToggle = new SimilarEdgeHighlightToggle(document.getElementById('similar_edge_highlight') as HTMLInputElement, editor);
    const constraintHintToggle = new ConstraintHintToggle(document.getElementById('constraint_hint') as HTMLInputElement, editor);
    const statusLabel = new StatusLabel(document.getElementById('dislike')!, editor);
    const relayoutButton = new RelayoutButton(document.getElementById('run_relayout') as HTMLButtonElement, editor);

    editor.start();

    problemSelector.start();
    dashboardSaver.start();
    fragmentUpdater.start();
    outputTextArea.start();
    zoomSlider.start();
    similarEdgeHighlightToggle.start();
    constraintHintToggle.start();
    statusLabel.start();
    relayoutButton.start();

    let problemId = 1;
    let baseSolutionId = null;
    let basePose = null;
    if (window.location.hash) {
        const url = new URL(window.location.hash.substring(1), 'http://example.com');
        const pidParam = url.searchParams.get('problem_id');
        if (pidParam) {
            problemId = +pidParam;
        }
        const sidParam = url.searchParams.get('base_solution_id');
        if (sidParam) {
            baseSolutionId = +sidParam;
        }
        const poseParam = url.searchParams.get('pose');
        if (poseParam) {
            basePose = JSON.parse(poseParam);
        }
        window.location.hash = '';
    }

    await problemSelector.setProblemId(problemId);
    if (baseSolutionId) {
        await solutionSet.getSolution(baseSolutionId).then((solution: Solution) => {
            editor.setPose(solution.pose);
        });
    }
    if (basePose) {
        editor.setPose(basePose);
    }
}

init();
