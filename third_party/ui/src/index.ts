import {Editor} from './editor';
import {
    DistanceToggle,
    OutputTextArea,
    ProblemSelector,
    StatusLabel,
    ZoomSlider
} from './controls';
import {ProblemSet} from './problems';

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
