import {useParams} from 'react-router-dom';
import {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Link} from 'react-router-dom';

import {makeStyles} from '@material-ui/core/styles';
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
import EditIcon from '@material-ui/icons/Edit';
import Snackbar from '@material-ui/core/Snackbar';
import {Model} from './model';
import {Typography} from '@material-ui/core';
import {Viewer} from './editor/Viewer';
import {scoreInfo} from './utils';

const useStyles = makeStyles((theme) => ({
    buttons: {
        '& > *': {
            margin: theme.spacing(1),
        },
    },
}));

export interface SolutionPageProps {
    model: Model;
}

export const SolutionPage = (props: SolutionPageProps) => {
    const {model} = props;

    const classes = useStyles();
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
    }, [model, solutionID]);

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

    const si = scoreInfo(problem, solution);
    let diff = "";
    let scoreText = "";
    if (problem.minimal_dislike !== solution.dislike) {
        diff = ` (トップ ${problem.minimal_dislike} / ${solution.dislike - problem.minimal_dislike}点差)`
        scoreText = `${si.score} (最大 ${si.maxScore} / 残り ${si.maxScore - si.score} / ${Math.ceil(100 - si.ratio * 100)}%)`;
    } else {
        diff = " (トップタイ)"
        scoreText = `${si.score} (MAX)`;
    }
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
                            <TableCell>
                                <Link to={problemLink} style={{paddingRight: '10px'}}>{solution.problem_id}</Link>
                                <Button variant="contained" color="secondary" href={`https://poses.live/problems/${solution.problem_id}`}>公式のSubmit一覧を見る</Button>
                            </TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Created at</TableCell>
                            <TableCell>{createdAt.toString()}</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Dislike</TableCell>
                            <TableCell>{solution.dislike}{diff}</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Score</TableCell>
                            <TableCell>{scoreText}</TableCell>
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
                            <TableCell className={classes.buttons}>
                                <Button disabled={sending} color="primary" onClick={handleSend} endIcon={<SendIcon />} variant="contained">公式にSubmit</Button>
                                <Button color="secondary" href={`https://nya3jp.github.io/icfpc2021/fcc7938b3c545e6ff51b101ea86f548b/#?problem_id=${solution.problem_id}&base_solution_id=${solution.solution_id}`} endIcon={<EditIcon />} variant="contained">エディタで編集</Button>
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
