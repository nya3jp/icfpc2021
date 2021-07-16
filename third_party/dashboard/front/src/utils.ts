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

import {Problem, Solution, BonusMap} from './types';

export function pointRatio(problem: Problem, solution: Solution): number {
    return Math.sqrt((problem.minimal_dislike + 1) / (solution.dislike + 1));
};

export function maxScore(problem: Problem): number {
    return Math.ceil(1000.0
        * Math.log2((problem.data.figure.vertices.length
            * problem.data.figure.edges.length
            * problem.data.hole.length) / 6.0));
};

export interface ScoreInfo {
    maxScore: number;
    score: number;
    ratio: number;
};

export function scoreInfo(problem: Problem, solution: Solution): ScoreInfo {
    let ratio = pointRatio(problem, solution);
    let ms = maxScore(problem);
    let score = Math.ceil(ms * ratio);
    return {
        maxScore: ms,
        score: score,
        ratio: ratio,
    };
};

export function bonusMap(problems: Problem[]): BonusMap {
    let m: BonusMap = {};
    problems.forEach((problem) => {
        problem.data.bonuses.forEach((bonus) => {
            if (!m[bonus.problem]) {
                m[bonus.problem] = [];
            }
            m[bonus.problem].push({
                from_problem_id: problem.problem_id,
                kind: bonus.bonus,
            });
        });
    });
    return m;
};

export function numberWithCommas(x: number): string {
    return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
};
