import {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Model} from './model';
import {Link} from 'react-router-dom';

import Container from '@material-ui/core/Container';
import Grid from '@material-ui/core/Grid';
import Paper from '@material-ui/core/Paper';
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
    let color = "#FFF";
    if (solution.dislike === 0) {
        color = green[100];
    }
    return (
        <Link to={solutionLink}>
            <Grid container spacing={2} style={{background: color}}>
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

const ProblemList = (props: ProblemListProps) => {
    const {model} = props;

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

    if (problems.length === 0) return <p>No solutions</p>;

    return (
        <Container component={Paper}>
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
                        return (
                            <TableRow key={problem.problem_id}>
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
