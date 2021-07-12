import {useParams} from 'react-router-dom';
import {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Link} from 'react-router-dom';

import {makeStyles} from '@material-ui/core/styles';
import Button from '@material-ui/core/Button';
import ButtonGroup from '@material-ui/core/ButtonGroup';
import Paper from '@material-ui/core/Paper';
import DeleteIcon from '@material-ui/icons/Delete';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import Container from '@material-ui/core/Container';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import TextField from '@material-ui/core/TextField';
import {Model} from './model';
import {Typography} from '@material-ui/core';

import {Viewer} from './editor/Viewer';
import {scoreInfo} from './utils';
import {EditButton, OfficialSubmitButton, BonusChip} from './buttons';

const useStyles = makeStyles((theme) => ({
    buttons: {
        '& > *': {
            margin: theme.spacing(1),
        },
    },
}));

export interface SolutionPageProps {
    model: Model;
};

export const SolutionPage = (props: SolutionPageProps) => {
    const {model} = props;

    const classes = useStyles();
    const {solutionID} = useParams<{solutionID: string}>();
    const [solution, setSolution] = useState<Solution | null>(null);
    const [problem, setProblem] = useState<Problem | null>(null);
    const [newTag, setNewTag] = useState<string>("");

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

    const handleChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
        setNewTag(event.target.value);
    };
    const handleSubmit = async () => {
        setSolution(await model.setSolutionTag(+solutionID, newTag));
        setNewTag("");
    };
    const handleTagDelete = (tag: string) => {
        (async () => {
            await model.deleteSolutionTag(+solutionID, tag);
            setSolution(await model.getSolution(+solutionID));
        })();
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
    });

    return (
        <Container>
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
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">使用🍆</TableCell>
                            <TableCell>{
                                solution.data.bonuses != null &&
                                solution.data.bonuses.length === 1 &&
                                <BonusChip bonus={solution.data.bonuses[0].bonus} />
                            }</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">獲得🍆</TableCell>
                            <TableCell>{
                                solution.acquired_bonuses != null &&
                                solution.acquired_bonuses.map((bonus) =>
                                    <BonusChip key={`bonus-${bonus.bonus}`} bonus={bonus.bonus} />
                                )
                            }</TableCell>
                        </TableRow>

                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Tags</TableCell>
                            <TableCell>
                                {
                                    solution.tags.map((tag) => {
                                        return (
                                            <ButtonGroup color="primary" key={`tag=${tag}`} >
                                                <Button component={Link} to={`/tags/${tag}`} style={{textTransform: "none"}}>{tag}</Button>
                                                <Button onClick={() => handleTagDelete(tag)}><DeleteIcon /></Button>
                                            </ButtonGroup>
                                        );
                                    })
                                }
                                <form noValidate autoComplete="off" onSubmit={handleSubmit}>
                                    <TextField value={newTag} label="Add tag" onChange={handleChange} />
                                </form>
                            </TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">JSON</TableCell>
                            <TableCell><textarea style={{width: '100%', height: '200px'}}>{dump}</textarea></TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell></TableCell>
                            <TableCell className={classes.buttons}>
                                <OfficialSubmitButton model={model} solutionID={+solutionID} />
                                <EditButton problemID={solution.problem_id} solutionID={solution.solution_id} />
                            </TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </TableContainer>
        </Container>
    );
};
