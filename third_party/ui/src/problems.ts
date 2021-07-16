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

import {Problem} from './types';

const MAX_PROBLEM_ID: number = 132;

export class ProblemSet {
    constructor() {
    }

    public async getProblemIds(): Promise<number[]> {
        return Array.from(Array(MAX_PROBLEM_ID).keys()).map(num => num + 1);
    }

    public async getProblem(id: number): Promise<Problem> {
        const res = await fetch(`./problems/${id}.problem`);
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json();
    }
}
