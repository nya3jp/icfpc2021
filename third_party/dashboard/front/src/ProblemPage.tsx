import {useParams} from 'react-router-dom';
import React, {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Link} from 'react-router-dom';
import {Model} from './model';
import {Paper, Typography} from '@material-ui/core';
import TableContainer from '@material-ui/core/TableContainer';
import Table from '@material-ui/core/Table';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import TableCell from '@material-ui/core/TableCell';
import TableBody from '@material-ui/core/TableBody';
import {Viewer} from './editor/Viewer';

function ProblemPane({ problem }: {problem: Problem}) {
    return (
        <div>
            <Viewer problem={problem.data} />
        </div>
    );
}

function SolutionsTable({ solutions }: { solutions: Solution[] }) {
    return (
        <TableContainer component={Paper}>
            <Table size="small" aria-label="a dense table">
                <TableHead>
                    <TableRow>
                        <TableCell>SolutionID</TableCell>
                        <TableCell>Created at</TableCell>
                        <TableCell>Dislike</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {solutions.map((solution) => {
                        const link = `/solutions/${solution.solution_id}`;
                        const createdAt = new Date();
                        createdAt.setTime(solution.created_at * 1000);
                        return (
                            <TableRow>
                                <TableCell><Link to={link}>{solution.solution_id}</Link></TableCell>
                                <TableCell>{createdAt.toString()}</TableCell>
                                <TableCell>{solution.dislike}</TableCell>
                            </TableRow>
                        );
                    })}
                </TableBody>
            </Table>
        </TableContainer>
    );
}

export interface ProblemPageProps {
    model: Model;
}

export const ProblemPage = (props: ProblemPageProps) => {
    const {model} = props;
    const {problemID} = useParams<{ problemID: string }>();
    const [problem, setProblem] = useState<Problem | null>(null);
    const [solutions, setSolutions] = useState<Solution[] | null>(null);

    useEffect(() => {
        (async () => {
            setProblem(await model.getProblem(+problemID));
        })();
    }, []);
    useEffect(() => {
        (async () => {
            setSolutions(await model.getSolutionsForProblem(+problemID));
        })();
    }, []);

    if (!problem || !solutions) {
      return <div></div>;
    }

    const createdAt = new Date();
    createdAt.setTime(problem.created_at * 1000);
    return (
      <div>
        <Typography variant={'h3'}>Problem { problem.problem_id }</Typography>
        <ProblemPane problem={problem} />
        <Typography variant={'h4'}>Solutions</Typography>
        <SolutionsTable solutions={solutions} />
      </div>
    );
};
