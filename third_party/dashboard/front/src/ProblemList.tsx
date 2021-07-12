import {Problem, Solution, GotBonus} from './types';
import {Model} from './model';
import {Link} from 'react-router-dom';
import React from 'react';

import Grid from '@material-ui/core/Grid';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import Typography from '@material-ui/core/Typography';
import {green} from '@material-ui/core/colors';

import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import ListItemText from '@material-ui/core/ListItemText';

import {Viewer} from './editor/Viewer';
import {maxScore, scoreInfo, bonusMap} from './utils';
import {BonusChip, RunSolverButton} from './buttons';

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
                            return <span key={`bonus-recv-${problem.problem_id}-${bonus.from_problem_id}-${bonus.kind}`}>{bonus.from_problem_id}から{bonus.kind}<br /></span>
                        })}
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        獲得可🍆:<br />
                        {problem.data.bonuses.map((bonus) => {
                            return <span key={`bonus-${problem.problem_id}-${bonus.problem}-${bonus.bonus}`}>{bonus.problem}へ{bonus.bonus}<br /></span>
                        })}
                    </ListItemText>
                </ListItem>
            </List>
        </Grid>
    );
};

const SolutionCell = ({model, problem, solution, bonus, showViewer, possibleBonuses}: {model: Model, problem: Problem, solution?: Solution, bonus: string, showViewer: boolean, possibleBonuses: Map<string, number> | undefined}) => {
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
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        使用🍆:
                        {
                            solution.data.bonuses != null &&
                            solution.data.bonuses.length === 1 &&
                            <BonusChip bonus={solution.data.bonuses[0].bonus} text={solution.data.bonuses[0].bonus} />
                        }
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        獲得🍆:
                        {
                            solution.acquired_bonuses != null &&
                            solution.acquired_bonuses.map((bonus) =>
                                <BonusChip key={`bonus-${bonus.bonus}`} bonus={bonus.bonus} text={bonus.bonus}/>
                            )
                        }
                    </ListItemText>
                </ListItem>
                <ListItem divider={true} dense={true}>
                    <ListItemText>
                        獲得可能🍆:
                        {
                            possibleBonuses != null &&
                            Array.from(possibleBonuses.entries()).map((v) => {
                                console.log(v);
                                return <BonusChip key={`bonus-${v[0]}`} bonus={v[0]} text={v[1].toString()} />;
                            })
                        }
                    </ListItemText>
                </ListItem>
            </List>
        </Grid>
    );
};

export interface ListColumnData {
    header: string;
    bonus: string;
    solutions: Map<number, Solution>;
    possibleBonuses: Map<number, Map<string, number>> | undefined,
};

export interface ProblemListProps {
    model: Model;
    problems: Problem[];
    hiddenProblems: Set<number>;
    greenBackgroundProblems: Set<number>;
    columns: ListColumnData[];
    showViewer: boolean;
};

export const ProblemList = (props: ProblemListProps) => {
    const {model, problems, hiddenProblems, greenBackgroundProblems, columns, showViewer} = props;
    const bonuses = bonusMap(problems);

    return (
        <Table size="small">
            <TableHead>
                <TableRow>
                    <TableCell>ID</TableCell>
                    <TableCell>Problem</TableCell>
                    {columns.map((column) => <TableCell key={`header-${column.header}`}>{column.header}</TableCell>)}
                </TableRow>
            </TableHead>
            <TableBody>
                {problems.map((problem) => {
                    if (hiddenProblems.has(problem.problem_id)) {
                        return <React.Fragment></React.Fragment>
                    }
                    const problemLink = `/problems/${problem.problem_id}`;
                    let color = "#FFF";
                    if (greenBackgroundProblems.has(problem.problem_id)) {
                        color = green[100];
                    }
                    return (
                        <TableRow key={`problem-row-${problem.problem_id}`} style={{background: color}}>
                            <TableCell align="right"><Link to={problemLink} style={{textDecoration: 'none'}}><Typography variant="h2">{problem.problem_id}</Typography></Link></TableCell>
                            <TableCell style={{verticalAlign: 'top'}}>
                                <ProblemCell model={model} problem={problem} bonuses={bonuses[problem.problem_id]} showViewer={showViewer} />
                            </TableCell>
                            {columns.map((column) => {
                                const sol = column.solutions.get(problem.problem_id);
                                return (
                                    <TableCell key={`problem-row-${problem.problem_id}-column-${column.header}`} style={{verticalAlign: 'top'}}>
                                        <SolutionCell model={model} problem={problem} showViewer={showViewer} bonus={column.bonus} solution={sol}
                                            possibleBonuses={column.possibleBonuses?.get(problem.problem_id)}/>
                                    </TableCell>
                                );
                            })}
                        </TableRow>
                    );
                })}
            </TableBody>
        </Table>
    );
};
