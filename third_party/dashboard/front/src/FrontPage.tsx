import React, {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Model} from './model';
import {Link} from 'react-router-dom';

import Container from '@material-ui/core/Container';
import FormControlLabel from '@material-ui/core/FormControlLabel';
import FormGroup from '@material-ui/core/FormGroup';
import Grid from '@material-ui/core/Grid';
import Paper from '@material-ui/core/Paper';
import Switch from '@material-ui/core/Switch';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import {Viewer} from './editor/Viewer';
import {green} from '@material-ui/core/colors';


type SolutionsMap = {[key: number]: Solution[]};

interface ProblemListProps {
    model: Model;
}

const ProblemCell = ({problem}: {problem: Problem}) => {
    const problemLink = `/problems/${problem.problem_id}`;
    return (
        <Link to={problemLink}>
            <Grid container spacing={2}>
                <Grid item>
                    <Viewer problem={problem} size={100} />
                </Grid>
                <Grid item>
                    <Table size="small">
                        <TableBody>
                            <TableRow>
                                <TableCell>ProblemID</TableCell>
                                <TableCell>{problem.problem_id}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>Minimal Dislike</TableCell>
                                <TableCell>{problem.minimal_dislike}</TableCell>
                            </TableRow>
                        </TableBody>
                    </Table>
                </Grid>
            </Grid>
        </Link>
    );
};

const SolutionCell = ({problem, solution}: {problem: Problem, solution: Solution}) => {
    const solutionLink = `/solutions/${solution.solution_id}`;
    return (
        <Link to={solutionLink}>
            <Grid container spacing={2}>
                <Grid item>
                    <Viewer problem={problem} solution={solution} size={100} />
                </Grid>
                <Grid item>
                    <Table size="small">
                        <TableBody>
                            <TableRow>
                                <TableCell>SolutionID</TableCell>
                                <TableCell>{solution.solution_id}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>Dislike</TableCell>
                                <TableCell>{solution.dislike}</TableCell>
                            </TableRow>
                        </TableBody>
                    </Table>
                </Grid>
            </Grid>
        </Link>
    );
};

interface FormFilterState {
    hideTopTie: boolean;
};

const ProblemList = (props: ProblemListProps) => {
    const {model} = props;

    const [formFilter, setFormFilter] = useState<FormFilterState>({
        hideTopTie: false,
    });
    const [problems, setProblems] = useState<Problem[]>([]);
    const [solutions, setSolutions] = useState<SolutionsMap>({});

    useEffect(() => {
        // Every time the state is updated, this is called...
        if (problems.length === 0) {
            model.getProblems().then((ps: Problem[]) => setProblems(ps));
        } else if (Object.keys(solutions).length === 0) {
            problems.forEach((problem: Problem) => {
                model.getSolutionsForProblem(problem.problem_id)
                    .then((ss: Solution[]) => {
                        setSolutions((solutions: SolutionsMap) => {
                            let m = {
                                ...solutions,
                            };
                            m[problem.problem_id] = ss;
                            return m;
                        });
                    });
            });
        }
    });

    const switchHideTopTie = (event: React.ChangeEvent<HTMLInputElement>) => {
        setFormFilter({...formFilter, hideTopTie: event.target.checked});
    };

    if (problems.length === 0) return <p>No solutions</p>;

    return (
        <Container component={Paper}>
            <FormGroup row>
                <FormControlLabel
                    control={
                        <Switch
                            checked={formFilter.hideTopTie}
                            onChange={switchHideTopTie}
                            color="primary"
                        />
                    }
                    label="Hide top tie problems"
                />
            </FormGroup>
            <Table size="small">
                <TableHead>
                    <TableRow>
                        <TableCell>Problem</TableCell>
                        <TableCell>Best Solution</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {problems.map((problem) => {
                        const ss = solutions[problem.problem_id];
                        if (!ss || ss.length === 0) {
                            return (
                                <TableRow key={problem.problem_id}>
                                    <TableCell><ProblemCell problem={problem} /></TableCell>
                                    <TableCell></TableCell>
                                </TableRow>
                            );
                        }
                        const sol = ss.reduce((prev, current) => {
                            return prev.dislike < current.dislike ? prev : current;
                        });
                        const topTie = sol.dislike === problem.minimal_dislike;
                        if (formFilter.hideTopTie && topTie) {
                            return <div></div>;
                        }
                        let color = "#FFF";
                        if (topTie) {
                            color = green[100];
                        }
                        return (
                            <TableRow key={problem.problem_id} style={{background: color}}>
                                <TableCell><ProblemCell problem={problem} /></TableCell>
                                <TableCell><SolutionCell problem={problem} solution={sol} /></TableCell>
                            </TableRow>
                        );
                    })}
                </TableBody>
            </Table>
        </Container>
    );
};

export interface FrontPageProps {
    model: Model;
}

export const FrontPage = (props: FrontPageProps) => {
    const {model} = props;

    return (
        <ProblemList model={model} />
    );
};
