import React, {useEffect, useState} from 'react';
import {Solution} from './types';
import {Link} from 'react-router-dom';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import {Model} from './model';

interface RecentSolutionsListProps {
    solutions: Solution[];
}

const RecentSolutionsList = (props: RecentSolutionsListProps) => {
    const {solutions} = props;
    return (
        <div>
            <TableContainer component={Paper}>
                <Table size="small" aria-label="a dense table">
                    <TableHead>
                        <TableRow>
                            <TableCell>ProblemID</TableCell>
                            <TableCell>SolutionID</TableCell>
                            <TableCell>Created at</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {solutions.map((solution) => {
                            const link = `/problems/${solution.problem_id}/solutions/${solution.solution_id}`;
                            const createdAt = new Date();
                            createdAt.setTime(solution.created_at * 1000);
                            return (
                                <TableRow>
                                    <TableCell>{solution.problem_id}</TableCell>
                                    <TableCell><Link to={link}>{solution.solution_id.substring(0, 8)}...</Link></TableCell>
                                    <TableCell>{createdAt.toString()}</TableCell>
                                </TableRow>
                            );
                        })}
                    </TableBody>
                </Table>
            </TableContainer>
        </div>
    );
};

export interface FrontPageProps {
    model: Model;
}

interface FrontPageState {
    currList: Solution[];
}

export const FrontPage = (props: FrontPageProps) => {
    const {model} = props;
    const solutions: Solution[] = [];

    /*
    // TODO: Use Model to retrieve a list of problems.
    const [appState, setAppState] = useState<FrontPageState>({
        currList: [],
    });

    useEffect(() => {
        setAppState({currList: []});
        fetch(`//localhost:8080/api/solutions/highscore`)
            .then((res) => res.json())
            .then((ss: Solution[]) => {
                setAppState({currList: ss});
                ss.map((solution) => {
                    ensureSolution(solution.problem_id, solution.solution_id);
                });
            });
    }, [setAppState]);
    */

    return (
        <div>
            <h2></h2>
            <RecentSolutionsList solutions={solutions} />
        </div>
    );
};
