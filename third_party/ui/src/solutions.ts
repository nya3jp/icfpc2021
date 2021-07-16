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

import {Solution, Pose} from './types';

export class SolutionSet {
    constructor() {
    }

    public async getSolution(id: number): Promise<Solution> {
        const res = await fetch(`https://spweek.badalloc.com/api/solutions/${id}`);
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json().then((m: any) => {
            return {
                id: m.solution_id,
                pose: m.data.vertices,
            } as Solution;
        });
    }

    public async addSolution(problemId: number, pose: Pose): Promise<number> {
        const res = await fetch(`https://spweek.badalloc.com/api/problems/${problemId}/solutions`, {
            method: "post",
            body: JSON.stringify({vertices: pose}),
        });
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json().then((m: any) => {
            return m.solution_id;
        });
    }
}
