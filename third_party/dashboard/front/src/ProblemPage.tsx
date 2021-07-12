import {useEffect, useState} from 'react';
import {useParams} from 'react-router-dom';

import Typography from '@material-ui/core/Typography';
import Container from '@material-ui/core/Container';

import {Problem, Solution} from './types';
import {Model} from './model';
import {Viewer} from './editor/Viewer';
import {ProblemSolutionPair, SolutionsTable} from './SolutionsTable';

function ProblemPane({problem}: {problem: Problem}) {
    return (
        <div>
            <Viewer problem={problem} />
        </div>
    );
}

export interface ProblemPageProps {
    model: Model;
}

export const ProblemPage = (props: ProblemPageProps) => {
    const {model} = props;
    const {problemID} = useParams<{problemID: string}>();
    const [problem, setProblem] = useState<Problem | null>(null);
    const [solutions, setSolutions] = useState<Solution[] | null>(null);

    useEffect(() => {
        (async () => {
            setProblem(await model.getProblem(+problemID));
        })();
    }, [model, problemID]);
    useEffect(() => {
        (async () => {
            let solutions = await model.getSolutionsForProblem(+problemID);
            console.log(solutions);
            solutions = solutions.sort((s1: Solution, s2: Solution) => {
                if (s1.dislike < s2.dislike) {
                    return -1;
                } else if (s2.dislike < s1.dislike) {
                    return 1;
                }
                if (s1.created_at < s2.created_at) {
                    return -1;
                } else if (s2.created_at < s1.created_at) {
                    return 1;
                }
                return 0;
            });
            setSolutions(solutions);
        })();
    }, [model, problemID]);

    if (!problem || !solutions) {
        return <div></div>;
    }

    const pairs: ProblemSolutionPair[] = solutions.map((solution) => {
        return {
            problem: problem,
            solution: solution,
        };
    });

    return (
        <Container>
            <Typography variant={'h3'}>Problem {problem.problem_id}</Typography>
            <ProblemPane problem={problem} />

            <Typography variant={'h4'}>Solutions</Typography>
            <SolutionsTable pairs={pairs} />
        </Container>
    );
};
