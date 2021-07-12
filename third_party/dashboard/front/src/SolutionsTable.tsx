import {Link} from 'react-router-dom';

import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Typography from '@material-ui/core/Typography';

import {Problem, Solution} from './types';
import {Viewer} from './editor/Viewer';
import {BonusChip} from './buttons';

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
                        const problemLink = `/problems/${problem.problem_id}`;
                        const link = `/solutions/${solution.solution_id}`;
                        const createdAt = new Date();
                        createdAt.setTime(solution.created_at * 1000);
                        return (
                            <TableRow key={solution.solution_id}>
                                {showProblem && <TableCell align="right"><Link to={problemLink} style={{textDecoration: 'none'}}><Typography variant="h2">{problem.problem_id}</Typography></Link></TableCell>}
                                {showProblem && <TableCell><Link to={problemLink}><Viewer problem={problem} size={100} /></Link></TableCell>}
                                <TableCell><Link to={link}>{solution.solution_id}</Link></TableCell>
                                <TableCell><Link to={link}><Viewer problem={problem} solution={solution} size={100} /></Link></TableCell>
                                <TableCell>{createdAt.toString()}</TableCell>
                                <TableCell>{solution.dislike}</TableCell>
                                <TableCell>{
                                    solution.data.bonuses != null &&
                                    solution.data.bonuses.length === 1 &&
                                    <BonusChip bonus={solution.data.bonuses[0].bonus} text={solution.data.bonuses[0].bonus}/>
                                }</TableCell>
                                <TableCell>{
                                    solution.acquired_bonuses != null &&
                                    solution.acquired_bonuses.map((bonus) =>
                                        <BonusChip key={`bonus-${bonus.bonus}`} bonus={bonus.bonus} text={bonus.bonus} />
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
