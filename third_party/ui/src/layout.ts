/**
 * Copyright 2021 Team Special Weekend
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import {Edge, Pose} from './types';

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
    const result: Pose = [];
    for (let i = 0; i < pose.length; ++i) {
        const pos = layout.getNodePosition(i);
        result.push([pos.x, pos.y]);
    }
    return result;
}
