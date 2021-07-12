import {useEffect, useState} from 'react';
import {useParams} from 'react-router-dom';

import Typography from '@material-ui/core/Typography';
import Container from '@material-ui/core/Container';

import {Problem, Solution} from './types';
import {Model} from './model';
import {ProblemSolutionPair, SolutionsTable} from './SolutionsTable';

export const TagPage = ({model}: {model: Model}) => {
    const {tag} = useParams<{tag: string}>();
    const [problems, setProblems] = useState<Map<number, Problem>>(new Map<number, Problem>());
    const [solutions, setSolutions] = useState<Solution[]>([]);

    useEffect(() => {
        (async () => {
            const solutions = await model.getSolutionsForTag(tag);
            setSolutions(solutions);
            solutions.forEach(async (solution) => {
                const problem = await model.getProblem(solution.problem_id);
                setProblems((problems) => {
                    const m = new Map<number, Problem>(problems);
                    m.set(problem.problem_id, problem);
                    return m;
                });
            });
        })();
    }, [model, tag]);

    if (!solutions) {
        return <div></div>;
    }

    const pairs: ProblemSolutionPair[] = [];
    solutions.forEach((solution) => {
        const problem = problems.get(solution.problem_id);
        if (problem) {
            pairs.push({
                problem: problem,
                solution: solution,
            });
        }
    });

    return (
        <Container>
            <Typography variant={'h3'}>Solutions with tag: {tag}</Typography>
            <SolutionsTable pairs={pairs} showProblem />
        </Container>
    );
};
