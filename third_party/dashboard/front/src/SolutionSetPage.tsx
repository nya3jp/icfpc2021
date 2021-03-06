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

import {useEffect, useState} from 'react';
import Container from '@material-ui/core/Container';
import {useLocation} from "react-router-dom";

import {Problem, Solution} from './types';
import {Model} from './model';
import {ListColumnData, ProblemList} from './ProblemList';
import {scoreInfo, numberWithCommas} from './utils';

export const SolutionSetPage = ({model}: {model: Model}) => {
    const [problems, setProblems] = useState<Problem[]>([]);
    const [solutions, setSolutions] = useState<Map<number, Solution>>(new Map<number, Solution>());
    let solutionIDs: number[] = [];
    const idStr = new URLSearchParams(useLocation().search).get("id");
    if (idStr) {
        solutionIDs = idStr.split(',').map((s) => +s)
    }

    useEffect(() => {
        // Every time the state is updated, this is called...
        if (problems.length === 0) {
            model.getProblems().then((ps: Problem[]) => {
                setProblems(ps);
            });
        } else if (solutions.size === 0) {
            solutionIDs.forEach((solutionID) => {
                model.getSolution(solutionID)
                    .then((s: Solution) => {
                        setSolutions((solutions: Map<number, Solution>) => {
                            let m = new Map<number, Solution>(solutions);
                            m.set(s.problem_id, s);
                            return m;
                        });
                    });
            });
        }
    });

    if (solutionIDs.length === 0) return <p>Specify /solutionsets?id=1,2,3</p>;
    if (problems.length === 0 || solutionIDs.length !== solutions.size) return <p>Loading...</p>;

    const point = problems.map((problem) => {
        const solution = solutions.get(problem.problem_id);
        if (!solution) {
            return 0;
        }
        const si = scoreInfo(problem, solution);
        return si.score;
    }).reduce((a, b) => a + b);

    const ps = problems.sort((p1: Problem, p2: Problem) => {
        return p1.problem_id - p2.problem_id;
    });

    const columns: ListColumnData[] = [
        {
            header: `Solution (Total ${numberWithCommas(point)})`,
            bonus: "",
            solutions: solutions,
            possibleBonuses: undefined,
        },
    ];

    let hiddenProblems: Set<number> = new Set<number>();
    let greenBackgroundProblems: Set<number> = new Set<number>();
    return (
        <Container>
            <ProblemList model={model} problems={ps} hiddenProblems={hiddenProblems}
                greenBackgroundProblems={greenBackgroundProblems} columns={columns}
                showViewer={true} />
        </Container>
    );
};
