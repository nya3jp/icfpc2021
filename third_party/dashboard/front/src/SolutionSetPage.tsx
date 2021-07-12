import {useEffect, useState} from 'react';
import Container from '@material-ui/core/Container';
import {useLocation} from "react-router-dom";

import {Problem, Solution} from './types';
import {Model} from './model';
import {ListColumnData, ProblemList} from './ProblemList';

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
    if (problems.length === 0) return <p>Loading...</p>;

    const ps = problems.sort((p1: Problem, p2: Problem) => {
        return p1.problem_id - p2.problem_id;
    });

    const columns: ListColumnData[] = [
        {
            header: "Solution",
            bonus: "",
            solutions: solutions,
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
