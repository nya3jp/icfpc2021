import {useEffect, useState} from 'react';
import Container from '@material-ui/core/Container';
import {useLocation} from "react-router-dom";

import {Problem, Solution} from './types';
import {Model} from './model';
import {ListColumnData, ProblemList} from './ProblemList';
import {scoreInfo, numberWithCommas} from './utils';

export const TagComparePage = ({model}: {model: Model}) => {
    const [inited, setInited] = useState<boolean>(false);
    const [problems, setProblems] = useState<Problem[]>([]);
    const [solutions, setSolutions] = useState<Map<string, Solution[]>>(new Map<string, Solution[]>());
    let tags: string[] = [];
    const tagStr = new URLSearchParams(useLocation().search).get("tag");
    if (tagStr) {
        tags = tagStr.split(',');
    }

    useEffect(() => {
        if (inited) {
            return;
        }
        setInited(true);
        if (problems.length === 0) {
            model.getProblems().then((ps) => setProblems(ps));
        }
        tags.forEach(async (tag) => {
            const ss = await model.getSolutionsForTag(tag)
            setSolutions((solutions: Map<string, Solution[]>) => {
                let m = new Map<string, Solution[]>(solutions);
                m.set(tag, ss);
                return m;
            });
        })
    });

    if (tags.length === 0) return <p>Specify /compare?tag=tag1,tag2,tag3</p>;
    if (problems.length === 0 || tags.length !== solutions.size) return <p>Loading...</p>;

    const columns: ListColumnData[] = [];
    tags.forEach((tag) => {
        const ss = solutions.get(tag);
        if (!ss) {
            return;
        }
        const m = new Map<number, Solution>();
        ss.forEach((s) => {m.set(s.problem_id, s)});
        const point = problems.map((problem) => {
            const solution = m.get(problem.problem_id);
            if (!solution) {
                return 0;
            }
            const si = scoreInfo(problem, solution);
            return si.score;
        }).reduce((a, b) => a + b);

        columns.push({
            header: `${tag} (Total ${numberWithCommas(point)})`,
            bonus: "",
            solutions: m,
            possibleBonuses: undefined,
        });
    });

    const ps = problems.sort((p1: Problem, p2: Problem) => {
        return p1.problem_id - p2.problem_id;
    });

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
