import React, {useEffect, useState} from 'react';
import {Problem, Solution, GotBonus, BonusMap} from './types';
import {Model} from './model';
import {Link} from 'react-router-dom';

import Container from '@material-ui/core/Container';
import FormControl from '@material-ui/core/FormControl';
import FormControlLabel from '@material-ui/core/FormControlLabel';
import FormGroup from '@material-ui/core/FormGroup';
import Grid from '@material-ui/core/Grid';
import InputLabel from '@material-ui/core/InputLabel';
import MenuItem from '@material-ui/core/MenuItem';
import Select from '@material-ui/core/Select';
import Switch from '@material-ui/core/Switch';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Typography from '@material-ui/core/Typography';
import {green} from '@material-ui/core/colors';
import {makeStyles} from '@material-ui/core/styles';

import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import ListItemText from '@material-ui/core/ListItemText';

import {Viewer} from './editor/Viewer';
import {maxScore, scoreInfo, bonusMap} from './utils';
import {RunSolverButton} from './buttons';

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

const CondViewer = ({problem, solution, showViewer}: {problem: Problem, solution?: Solution, showViewer: boolean}) => {
    if (!showViewer) {
        return <div></div>
    }
    if (!solution) {
        return <Viewer problem={problem} size={100} />
    }
    return <Viewer problem={problem} solution={solution} size={100} />
};

const ProblemCell = ({model, problem, bonuses, showViewer}: {model: Model, problem: Problem, bonuses: GotBonus[], showViewer: boolean}) => {
    if (!bonuses) {
        bonuses = [];
    }
    return (
        <Grid container direction="column" alignItems="center">
            <RunSolverButton model={model} problem={problem} bonus="" text="ソルバ" />
            <CondViewer problem={problem} showViewer={showViewer} />
            <List dense={true} style={{width: '13em'}}>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        最小Dislike: <b>{problem.minimal_dislike}</b>
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        最大スコア: <b>{maxScore(problem)}</b>
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        使用可🍆:<br />
                        {bonuses.map((bonus) => {
                            return <span>{bonus.from_problem_id}から{bonus.kind}<br /></span>
                        })}
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        獲得可🍆:<br />
                        {problem.data.bonuses.map((bonus) => {
                            return <span>{bonus.problem}へ{bonus.bonus}<br /></span>
                        })}
                    </ListItemText>
                </ListItem>
            </List>
        </Grid>
    );
};

const SolutionCell = ({model, problem, solution, bonus, showViewer}: {model: Model, problem: Problem, solution?: Solution, bonus: string, showViewer: boolean}) => {
    if (!solution) {
        return (
            <Grid container direction="column" alignItems="center">
                <RunSolverButton model={model} problem={problem} bonus={bonus} text="ソルバ" />
            </Grid>
        );
    }
    const solutionLink = `/solutions/${solution.solution_id}`;
    const si = scoreInfo(problem, solution);

    let dislikeText = "";
    let scoreText = "";
    if (problem.minimal_dislike !== solution.dislike) {
        dislikeText = `${solution.dislike} (${solution.dislike - problem.minimal_dislike}点差)`
        scoreText = `(残り ${si.maxScore - si.score} / ${Math.ceil(100 - si.ratio * 100)}%)`;
    } else {
        dislikeText = `${solution.dislike} (トップタイ)`
        scoreText = `(MAX)`;
    }
    return (
        <Grid container direction="column" alignItems="center">
            <RunSolverButton model={model} problem={problem} bonus={bonus} text="ソルバ" />
            <Link to={solutionLink} style={{textDecoration: 'none'}}>
                <CondViewer problem={problem} solution={solution} showViewer={showViewer} />
            </Link>
            <List dense={true} style={{width: '17em'}}>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        Dislike: <b>{dislikeText}</b>
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        スコア: <b>{si.score}<br />{scoreText}</b>
                    </ListItemText>
                </ListItem>
            </List>
        </Grid>
    );
};

interface FormFilterState {
    hideTopTie: boolean;
    hideZeroScore: boolean;
    hideViewer: boolean;
    order: string;
};

const ProblemList = (props: ProblemListProps) => {
    const {model} = props;
    const classes = useStyles();
    const ssKey = "ProblemListFormFilterState";

    const [formFilter, setFormFilter] = useState<FormFilterState>(() => {
        const s = localStorage.getItem(ssKey);
        if (s) {
            return JSON.parse(s);
        }
        return {
            hideTopTie: false,
            hideZeroScore: false,
            hideViewer: false,
            order: "ProblemID",
        };
    });
    const [problems, setProblems] = useState<Problem[]>([]);
    const [bonuses, setBonuses] = useState<BonusMap>([]);
    const [solutions, setSolutions] = useState<SolutionsMap>({});

    useEffect(() => {
        // Every time the state is updated, this is called...
        if (problems.length === 0) {
            model.getProblems().then((ps: Problem[]) => {
                setBonuses(bonusMap(ps));
                setProblems(ps);
            });
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

    const formFilterSaveHook = (d: FormFilterState) => {
        localStorage.setItem(ssKey, JSON.stringify(d));
    };
    const switchHideTopTie = (event: React.ChangeEvent<HTMLInputElement>) => {
        const v = {...formFilter, hideTopTie: event.target.checked};
        setFormFilter(v);
        formFilterSaveHook(v);
    };
    const switchHideZeroScore = (event: React.ChangeEvent<HTMLInputElement>) => {
        const v = {...formFilter, hideZeroScore: event.target.checked};
        setFormFilter(v);
        formFilterSaveHook(v);
    };
    const switchHideViewer = (event: React.ChangeEvent<HTMLInputElement>) => {
        const v = {...formFilter, hideViewer: event.target.checked};
        setFormFilter(v);
        formFilterSaveHook(v);
    };
    const switchSortOrder = (event: React.ChangeEvent<{value: unknown}>) => {
        const v = {...formFilter, order: event.target.value as string};
        setFormFilter(v);
        formFilterSaveHook(v);
    };

    if (problems.length === 0) return <p>No solutions</p>;

    let bestSolutions: BestSolutionMap = {};
    problems.forEach((problem) => {
        let ss = solutions[problem.problem_id];
        if (!ss) {
            return;
        }

        if (ss.length == 0) {
            return;
        }

        const sol = ss.reduce((prev, current) => {
            return prev.dislike < current.dislike ? prev : current;
        });
        bestSolutions[problem.problem_id] = sol;
    });

    const filteredBestSolutionMap = (bonus: string) => {
        let solutionMap: BestSolutionMap = {};
        problems.forEach((problem) => {
            let ss = solutions[problem.problem_id];
            if (!ss) {
                return;
            }

            ss = ss.filter((s) => {
                return s.data.bonuses != null && s.data.bonuses.some((b) => b.bonus == bonus);
            });

            if (ss.length == 0) {
                return;
            }

            const sol = ss.reduce((prev, current) => {
                return prev.dislike < current.dislike ? prev : current;
            });
            solutionMap[problem.problem_id] = sol;
        });
        return solutionMap;
    };
    const bestSolutionsGlobalist: BestSolutionMap = filteredBestSolutionMap("GLOBALIST");
    const bestSolutionsSuperflex: BestSolutionMap = filteredBestSolutionMap("SUPERFLEX");

    const ps = problems.sort((p1: Problem, p2: Problem) => {
        if (formFilter.order === "ProblemID") {
            return p1.problem_id - p2.problem_id;
        }
        let rem1 = 9999999;
        let rem2 = 9999999;
        if (formFilter.order === "HighRemainingScore") {
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
        } else if (formFilter.order === "HighRemainingScoreRatio") {
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
        if (rem1 !== rem2) {
            return rem2 - rem1;
        }
        return p1.problem_id - p2.problem_id;
    });

    return (
        <Container>
            <FormGroup row>
                <FormControlLabel control={<Switch checked={formFilter.hideTopTie} onChange={switchHideTopTie} color="primary" />} label="トップタイの問題を隠す" />
                <FormControlLabel control={<Switch checked={formFilter.hideZeroScore} onChange={switchHideZeroScore} color="primary" />} label="0点の問題を隠す" />
                <FormControlLabel control={<Switch checked={formFilter.hideViewer} onChange={switchHideViewer} color="primary" />} label="ビューアーを隠す" />
                <div className={classes.spacer}></div>
                <FormControl>
                    <InputLabel shrink id="sort-order-label">ソート順</InputLabel>
                    <Select labelId="sort-order-label" id="sort-order" value={formFilter.order} onChange={switchSortOrder}>
                        <MenuItem value={"ProblemID"}>Problem ID</MenuItem>
                        <MenuItem value={"HighRemainingScore"}>スコア伸びしろ多い順</MenuItem>
                        <MenuItem value={"HighRemainingScoreRatio"}>スコア伸びしろ(比率)多い順</MenuItem>
                    </Select>
                </FormControl>
            </FormGroup>
            <Table size="small">
                <TableHead>
                    <TableRow>
                        <TableCell>ID</TableCell>
                        <TableCell>Problem</TableCell>
                        <TableCell>Best Solution</TableCell>
                        <TableCell>Best (+GLOBALIST)</TableCell>
                        <TableCell>Best (+SUPERFLEX)</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {ps.map((problem) => {
                        const sol = bestSolutions[problem.problem_id];
                        const solGlobalist = bestSolutionsGlobalist[problem.problem_id];
                        const solSuperflex = bestSolutionsSuperflex[problem.problem_id];
                        const problemLink = `/problems/${problem.problem_id}`;
                        if (!sol) {
                            return (
                                <TableRow key={problem.problem_id}>
                                    <TableCell align="right"><Link to={problemLink} style={{textDecoration: 'none'}}><Typography variant="h2">{problem.problem_id}</Typography></Link></TableCell>
                                    <TableCell style={{verticalAlign: 'top'}}><ProblemCell model={model} problem={problem} bonuses={bonuses[problem.problem_id]} showViewer={!formFilter.hideViewer} /></TableCell>
                                    <TableCell style={{verticalAlign: 'top'}}><SolutionCell model={model} problem={problem} showViewer={!formFilter.hideViewer} bonus="" /></TableCell>
                                    <TableCell style={{verticalAlign: 'top'}}><SolutionCell model={model} problem={problem} showViewer={!formFilter.hideViewer} bonus="GLOBALIST" /></TableCell>
                                    <TableCell style={{verticalAlign: 'top'}}><SolutionCell model={model} problem={problem} showViewer={!formFilter.hideViewer} bonus="SUPERFLEX" /></TableCell>
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
                                <TableCell align="right"><Link to={problemLink} style={{textDecoration: 'none'}}><Typography variant="h2">{problem.problem_id}</Typography></Link></TableCell>
                                <TableCell style={{verticalAlign: 'top'}}><ProblemCell model={model} problem={problem} bonuses={bonuses[problem.problem_id]} showViewer={!formFilter.hideViewer} /></TableCell>
                                <TableCell style={{verticalAlign: 'top'}}><SolutionCell model={model} problem={problem} showViewer={!formFilter.hideViewer} bonus="" solution={sol} /></TableCell>
                                <TableCell style={{verticalAlign: 'top'}}><SolutionCell model={model} problem={problem} showViewer={!formFilter.hideViewer} bonus="GLOBALIST" solution={solGlobalist} /></TableCell>
                                <TableCell style={{verticalAlign: 'top'}}><SolutionCell model={model} problem={problem} showViewer={!formFilter.hideViewer} bonus="SUPERFLEX" solution={solSuperflex} /></TableCell>
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
