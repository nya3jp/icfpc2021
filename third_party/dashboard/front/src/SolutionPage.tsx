import {useParams} from 'react-router-dom';
import {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Link} from 'react-router-dom';

import Button from '@material-ui/core/Button';
import Chip from '@material-ui/core/Chip';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import SendIcon from '@material-ui/icons/Send';
import Snackbar from '@material-ui/core/Snackbar';
import {Model} from './model';
import {Typography} from '@material-ui/core';
import {Viewer} from './editor/Viewer';

export interface SolutionPageProps {
    model: Model;
}

export const SolutionPage = (props: SolutionPageProps) => {
    const {model} = props;
    const {solutionID} = useParams<{solutionID: string}>();
    const [solution, setSolution] = useState<Solution | null>(null);
    const [problem, setProblem] = useState<Problem | null>(null);
    const [sending, setSending] = useState<boolean>(false);
    const [message, setMessage] = useState<string>("");
    const [openTimedMessage, setOpenTimedMessage] = useState<boolean>(false);
    const [timedMessage, setTimedMessage] = useState<string>("");

    useEffect(() => {
        (async () => {
            const solution = await model.getSolution(+solutionID);
            setSolution(solution);
            const problem = await model.getProblem(solution.problem_id);
            setProblem(problem);
        })();
    }, []);

    if (!solution || !problem) {
        return <div></div>
    }

    const handleSend = async () => {
        setMessage("Sending the solution...");
        setSending(true);
        let resp = await model.submitSolution(+solutionID);
        setSending(false);
        setTimedMessage(resp);
        setOpenTimedMessage(true);
    };
    const handleClose = () => {
        setOpenTimedMessage(false);
    };

    const problemLink = `/problems/${problem.problem_id}`;
    const createdAt = new Date();
    createdAt.setTime(solution.created_at * 1000);
    const dump = JSON.stringify({
        problem_id: solution.problem_id,
        ...solution.data
    })
    return (
        <div>
            <Typography variant={'h3'}>Solution {solutionID}</Typography>
            <Viewer problem={problem} solution={solution} />
            <TableContainer component={Paper}>
                <Table size="small" aria-label="a dense table">
                    <TableHead>
                        <TableRow>
                            <TableCell align="right">Key</TableCell>
                            <TableCell>Value</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">SolutionID</TableCell>
                            <TableCell>{solution.solution_id}</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">ProblemID</TableCell>
                            <TableCell><Link to={problemLink}>{solution.problem_id}</Link></TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Created at</TableCell>
                            <TableCell>{createdAt.toString()}</TableCell>
                        </TableRow>
                        {solution.tags &&
                            <TableRow>
                                <TableCell component="th" scope="row" align="right">Tags</TableCell>
                                <TableCell>{solution.tags.map((tag) => <Link to={`/tags/${tag}`}><Chip label={tag} /></Link>)}</TableCell>
                            </TableRow>
                        }
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">JSON</TableCell>
                            <TableCell><textarea style={{width: '100%', height: '200px'}}>{dump}</textarea></TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell></TableCell>
                            <TableCell>
                                <Button disabled={sending} color="primary" onClick={handleSend} endIcon={<SendIcon />} variant="contained">公式にSubmit</Button>
                            </TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </TableContainer>
            <Snackbar open={sending} message={message} />
            <Snackbar open={openTimedMessage} autoHideDuration={3000} onClose={handleClose} message={timedMessage} />
        </div>
    );
};
