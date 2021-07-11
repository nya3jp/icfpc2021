import React, {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Model} from './model';
import {Link} from 'react-router-dom';

import {makeStyles} from '@material-ui/core/styles';
import Container from '@material-ui/core/Container';
import FormControl from '@material-ui/core/FormControl';
import FormControlLabel from '@material-ui/core/FormControlLabel';
import FormGroup from '@material-ui/core/FormGroup';
import Grid from '@material-ui/core/Grid';
import InputLabel from '@material-ui/core/InputLabel';
import MenuItem from '@material-ui/core/MenuItem';
import Paper from '@material-ui/core/Paper';
import Select from '@material-ui/core/Select';
import Switch from '@material-ui/core/Switch';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Typography from '@material-ui/core/Typography';
import {Viewer} from './editor/Viewer';
import {green} from '@material-ui/core/colors';
import {maxScore, scoreInfo} from './utils';

const useStyles = makeStyles((_) => ({
    spacer: {
        flexGrow: 1,
    },
}));


type SolutionsMap = {[key: number]: Solution[]};
type BestSolutionMap = {[key: number]: Solution};

interface ProblemListProps {
    model: Model;
}

const ProblemCell = ({problem}: {problem: Problem}) => {
    const problemLink = `/problems/${problem.problem_id}`;
    return (
        <Link to={problemLink} style={{ textDecoration: 'none' }}>
            <Grid container spacing={2}>
                <Grid item>
                    <Viewer problem={problem} size={100} />
                </Grid>
                <Grid item>
                    <Typography variant="h2">
                        {problem.problem_id}
                    </Typography>
                </Grid>
                <Grid item>
                    <Table size="small">
                        <TableBody>
                            <TableRow>
                                <TableCell>Minimal Dislike</TableCell>
                                <TableCell>{problem.minimal_dislike}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>Max Score</TableCell>
                                <TableCell>{maxScore(problem)}</TableCell>
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
    const si = scoreInfo(problem, solution);

    let diff = "";
    let scoreText = "";
    if (problem.minimal_dislike !== solution.dislike) {
        diff = ` (${solution.dislike - problem.minimal_dislike}点差)`
        scoreText = `${si.score} (残り ${si.maxScore - si.score} / ${Math.ceil(100 - si.ratio * 100)}%)`;
    } else {
        diff = " (トップタイ)"
        scoreText = `${si.score} (MAX)`;
    }
    return (
        <Link to={solutionLink} style={{ textDecoration: 'none' }}>
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
                                <TableCell>{solution.dislike}{diff}</TableCell>
                            </TableRow>
                            <TableRow>
                                <TableCell>Score</TableCell>
                                <TableCell>{scoreText}</TableCell>
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
    hideZeroScore: boolean;
};

const ProblemList = (props: ProblemListProps) => {
    const {model} = props;
    const classes = useStyles();

    const [formFilter, setFormFilter] = useState<FormFilterState>({
        hideTopTie: false,
        hideZeroScore: false,
    });
    const [problems, setProblems] = useState<Problem[]>([]);
    const [solutions, setSolutions] = useState<SolutionsMap>({});
    const [order, setOrder] = useState<string>("ProblemID");

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
    const switchHideZeroScore = (event: React.ChangeEvent<HTMLInputElement>) => {
        setFormFilter({...formFilter, hideZeroScore: event.target.checked});
    };
    const switchSortOrder = (event: React.ChangeEvent<{value: unknown}>) => {
        setOrder(event.target.value as string);
    };

    if (problems.length === 0) return <p>No solutions</p>;

    let bestSolutions: BestSolutionMap = {};
    problems.forEach((problem) => {
        const ss = solutions[problem.problem_id];
        if (!ss || ss.length === 0) {
            return;
        }
        const sol = ss.reduce((prev, current) => {
            return prev.dislike < current.dislike ? prev : current;
        });
        bestSolutions[problem.problem_id] = sol;
    });
    const ps = problems.sort((p1: Problem, p2: Problem) => {
        if (order === "ProblemID") {
            return p1.problem_id - p2.problem_id;
        }
        let rem1 = 9999999;
        let rem2 = 9999999;
        if (order === "HighRemainingScore") {
            const sol1 = bestSolutions[p1.problem_id];
            const sol2 = bestSolutions[p2.problem_id];
            if (sol1) {
                const si = scoreInfo(p1, sol1);
                rem1 = si.maxScore - si.score;
            } else {
                rem1 = maxScore(p1);
            }
            if (sol2) {
                const si = scoreInfo(p2, sol2);
                rem2 = si.maxScore - si.score;
            } else {
                rem2 = maxScore(p2);
            }
        } else if (order === "HighRemainingScoreRatio") {
            const sol1 = bestSolutions[p1.problem_id];
            const sol2 = bestSolutions[p2.problem_id];
            if (sol1) {
                const si = scoreInfo(p1, sol1);
                rem1 = 1 - si.ratio;
            } else {
                rem1 = maxScore(p1);
            }
            if (sol2) {
                const si = scoreInfo(p2, sol2);
                rem2 = 1 - si.ratio;
            } else {
                rem2 = maxScore(p2);
            }
        }
        if (rem1 != rem2) {
            return rem2 - rem1;
        }
        return p1.problem_id - p2.problem_id;
    });

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
                    label="トップタイの問題を隠す"
                />
                <FormControlLabel
                    control={
                        <Switch
                            checked={formFilter.hideZeroScore}
                            onChange={switchHideZeroScore}
                            color="primary"
                        />
                    }
                    label="0点の問題を隠す"
                />
                <div className={classes.spacer}></div>
                <FormControl>
                    <InputLabel shrink id="sort-order-label">ソート順</InputLabel>
                    <Select labelId="sort-order-label" id="sort-order" value={order} onChange={switchSortOrder}>
                        <MenuItem value={"ProblemID"}>Problem ID</MenuItem>
                        <MenuItem value={"HighRemainingScore"}>スコア伸びしろ多い順</MenuItem>
                        <MenuItem value={"HighRemainingScoreRatio"}>スコア伸びしろ(比率)多い順</MenuItem>
                    </Select>
                </FormControl>
            </FormGroup>
            <Table size="small">
                <TableHead>
                    <TableRow>
                        <TableCell>Problem</TableCell>
                        <TableCell>Best Solution</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {ps.map((problem) => {
                        const sol = bestSolutions[problem.problem_id];
                        if (!sol) {
                            return (
                                <TableRow key={problem.problem_id}>
                                    <TableCell><ProblemCell problem={problem} /></TableCell>
                                    <TableCell></TableCell>
                                </TableRow>
                            );
                        }
                        if (formFilter.hideZeroScore && sol.dislike === 0) {
                            return <div></div>;
                        }
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
