import {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Model} from './model';
import {Link} from 'react-router-dom';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import {Viewer} from './editor/Viewer';

type SolutionsMap = {[key: number]: Solution[]};

interface ProblemListProps {
    model: Model;
}

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
        <div>
            <TableContainer component={Paper}>
                <Table size="small" aria-label="a dense table">
                    <TableHead>
                        <TableRow>
                            <TableCell>ProblemID</TableCell>
                            <TableCell>Problem</TableCell>
                            <TableCell>Best SolutionID</TableCell>
                            <TableCell>Best Dislike</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        {problems.map((problem) => {
                            const ss = solutions[problem.problem_id];
                            const problemLink = `/problems/${problem.problem_id}`;
                            if (!ss || ss.length === 0) {
                                return (
                                    <TableRow key={problem.problem_id}>
                                        <TableCell><Link to={problemLink}>{problem.problem_id}</Link></TableCell>
                                        <TableCell><Link to={problemLink}><Viewer problem={problem} size={100} /></Link></TableCell>
                                        <TableCell></TableCell>
                                        <TableCell></TableCell>
                                    </TableRow>
                                );
                            }
                            const sol = ss.reduce((prev, current) => {
                                return prev.dislike < current.dislike ? prev : current;
                            });
                            const solutionLink = `/solutions/${sol.solution_id}`;
                            return (
                                <TableRow key={problem.problem_id}>
                                    <TableCell><Link to={problemLink}>{problem.problem_id}</Link></TableCell>
                                    <TableCell><Link to={problemLink}><Viewer problem={problem} size={100} /></Link></TableCell>
                                    <TableCell><Link to={solutionLink}>{sol.solution_id}</Link></TableCell>
                                    <TableCell>{sol.dislike}</TableCell>
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

export const FrontPage = (props: FrontPageProps) => {
    const {model} = props;

    return (
        <div>
            <ProblemList model={model} />
        </div>
    );
};
