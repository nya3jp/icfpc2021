import {Edge, Pose} from './types';
import {roundPoint} from './geom';

const createGraph = require('ngraph.graph');
const createLayout = require('ngraph.forcelayout');

export function forceLayout(pose: Pose, edges: Edge[], iters: number): Pose {
    const graph = createGraph();
    for (let i = 0; i < pose.length; ++i) {
        graph.addNode(i);
    }
    for (const [a, b] of edges) {
        graph.addLink(a, b);
    }
    const layout = createLayout(graph);
    for (let i = 0; i < pose.length; ++i) {
        layout.setNodePosition(i, ...pose[i]);
    }
    for (let i = 0; i < iters; ++i) {
        layout.step();
    }
    const result = [];
    for (let i = 0; i < pose.length; ++i) {
        const pos = layout.getNodePosition(i);
        result.push([pos.x, pos.y]);
    }
    return result;
}
