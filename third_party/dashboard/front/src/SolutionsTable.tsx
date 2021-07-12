import {Link} from 'react-router-dom';

import Chip from '@material-ui/core/Chip';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';

import {Problem, Solution} from './types';
import {Viewer} from './editor/Viewer';

export interface ProblemSolutionPair {
    problem: Problem;
    solution: Solution;
};

export function SolutionsTable({pairs, showProblem = false}: {pairs: ProblemSolutionPair[], showProblem?: boolean}) {
    return (
        <TableContainer component={Paper}>
            <Table size="small" aria-label="a dense table">
                <TableHead>
                    <TableRow>
                        {showProblem && <TableCell>ProblemID</TableCell>}
                        {showProblem && <TableCell>Problem</TableCell>}
                        <TableCell>SolutionID</TableCell>
                        <TableCell>Solution</TableCell>
                        <TableCell>Created at</TableCell>
                        <TableCell>Dislike</TableCell>
                        <TableCell>使用🍆</TableCell>
                        <TableCell>獲得🍆</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {pairs.map(({problem, solution}) => {
                        const link = `/solutions/${solution.solution_id}`;
                        const createdAt = new Date();
                        createdAt.setTime(solution.created_at * 1000);
                        return (
                            <TableRow key={solution.solution_id}>
                                {showProblem && <TableCell>{problem.problem_id}</TableCell>}
                                {showProblem && <TableCell><Viewer problem={problem} size={100} /></TableCell>}
                                <TableCell><Link to={link}>{solution.solution_id}</Link></TableCell>
                                <TableCell><Link to={link}><Viewer problem={problem} solution={solution} size={100} /></Link></TableCell>
                                <TableCell>{createdAt.toString()}</TableCell>
                                <TableCell>{solution.dislike}</TableCell>
                                <TableCell>{
                                    solution.data.bonuses != null &&
                                    solution.data.bonuses.length === 1 &&
                                    solution.data.bonuses[0].bonus
                                }</TableCell>
                                <TableCell>{
                                    solution.acquired_bonuses != null &&
                                    solution.acquired_bonuses.map((bonus) =>
                                        <Chip color="primary" key={`bonus-${bonus.bonus}`} label={bonus.bonus} />
                                    )
                                }</TableCell>
                            </TableRow>
                        );
                    })}
                </TableBody>
            </Table>
        </TableContainer>
    );
}
