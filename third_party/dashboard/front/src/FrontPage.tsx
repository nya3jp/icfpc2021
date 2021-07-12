import React, {useEffect, useState} from 'react';
import {Problem, Solution} from './types';
import {Model} from './model';

import Container from '@material-ui/core/Container';
import FormControl from '@material-ui/core/FormControl';
import FormControlLabel from '@material-ui/core/FormControlLabel';
import FormGroup from '@material-ui/core/FormGroup';
import InputLabel from '@material-ui/core/InputLabel';
import MenuItem from '@material-ui/core/MenuItem';
import Select from '@material-ui/core/Select';
import Switch from '@material-ui/core/Switch';
import {makeStyles} from '@material-ui/core/styles';

import {maxScore, scoreInfo} from './utils';
import {ListColumnData, ProblemList} from './ProblemList';

const useStyles = makeStyles((_) => ({
    spacer: {
        flexGrow: 1,
    },
}));

type BestSolutionMap = {[key: number]: Solution};
type SolutionsMap = {[key: number]: Solution[]};

interface FormFilterState {
    hideTopTie: boolean;
    hideZeroScore: boolean;
    hideViewer: boolean;
    order: string;
};

interface FrontPageProblemListProps {
    model: Model;
};

const FrontPageProblemList = (props: FrontPageProblemListProps) => {
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
    const [solutions, setSolutions] = useState<SolutionsMap>({});

    useEffect(() => {
        // Every time the state is updated, this is called...
        if (problems.length === 0) {
            model.getProblems().then((ps: Problem[]) => {
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

        ss = ss.filter((s) => {
            return s.data.bonuses == null || s.data.bonuses.length === 0;
        });

        if (ss.length === 0) {
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
                return s.data.bonuses != null && s.data.bonuses.some((b) => b.bonus === bonus);
            });

            if (ss.length === 0) {
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

    let sols: Map<number, Solution> = new Map<number, Solution>();
    let solGlobalists: Map<number, Solution> = new Map<number, Solution>();
    let solSuperflexes: Map<number, Solution> = new Map<number, Solution>();
    let hiddenProblems: Set<number> = new Set<number>();
    let greenBackgroundProblems: Set<number> = new Set<number>();
    let pbs: Map<number, Map<string, number>> = new Map<number, Map<string, number>>();
    let pbsGlobalists: Map<number, Map<string, number>> = new Map<number, Map<string, number>>();
    let pbsSuperflexes: Map<number, Map<string, number>> = new Map<number, Map<string, number>>();
    ps.forEach((problem) => {
        const sol = bestSolutions[problem.problem_id];
        const solGlobalist = bestSolutionsGlobalist[problem.problem_id];
        const solSuperflex = bestSolutionsSuperflex[problem.problem_id];
        if (sol) {
            sols.set(problem.problem_id, sol);
        }
        if (solGlobalist) {
            solGlobalists.set(problem.problem_id, solGlobalist);
        }
        if (solSuperflex) {
            solSuperflexes.set(problem.problem_id, solSuperflex);
        }

        if (sol) {
            if (formFilter.hideZeroScore && sol.dislike === 0) {
                hiddenProblems.add(problem.problem_id);
                return;
            }
            const topTie = sol.dislike === problem.minimal_dislike;
            if (formFilter.hideTopTie && topTie) {
                hiddenProblems.add(problem.problem_id);
                return;
            }
            if (topTie) {
                greenBackgroundProblems.add(problem.problem_id);
            }
        }

        pbs.set(problem.problem_id, new Map<string, number>());
        pbsGlobalists.set(problem.problem_id, new Map<string, number>());
        pbsSuperflexes.set(problem.problem_id, new Map<string, number>());
        if (solutions[problem.problem_id]) {
            solutions[problem.problem_id].map((s) => {
                if (s.data.bonuses == null || s.data.bonuses.length === 0) {
                    s.acquired_bonuses.map((v) => {
                        if (!pbs.get(problem.problem_id)?.has(v.bonus)) {
                            pbs.get(problem.problem_id)?.set(v.bonus, s.dislike)
                        }
                        const current = pbs.get(problem.problem_id)?.get(v.bonus);
                        if (current && current > s.dislike) {
                            pbs.get(problem.problem_id)?.set(v.bonus, s.dislike)
                        }
                    })
                }
                if (s.data.bonuses != null && s.data.bonuses.some((b) => b.bonus === "GLOBALIST")) {
                    s.acquired_bonuses.map((v) => {
                        if (!pbsGlobalists.get(problem.problem_id)?.has(v.bonus)) {
                            pbsGlobalists.get(problem.problem_id)?.set(v.bonus, s.dislike)
                        }
                        const current = pbsGlobalists.get(problem.problem_id)?.get(v.bonus);
                        if (current && current > s.dislike) {
                            pbsGlobalists.get(problem.problem_id)?.set(v.bonus, s.dislike)
                        }
                    })
                }
                if (s.data.bonuses != null && s.data.bonuses.some((b) => b.bonus === "SUPERFLEX")) {
                    s.acquired_bonuses.map((v) => {
                        if (!pbsSuperflexes.get(problem.problem_id)?.has(v.bonus)) {
                            pbsSuperflexes.get(problem.problem_id)?.set(v.bonus, s.dislike)
                        }
                        const current = pbsSuperflexes.get(problem.problem_id)?.get(v.bonus);
                        if (current && current > s.dislike) {
                            pbsSuperflexes.get(problem.problem_id)?.set(v.bonus, s.dislike)
                        }
                    })
                }
            })
        }
    });
    const columns: ListColumnData[] = [
        {
            header: "Best Solution",
            bonus: "",
            solutions: sols,
            possibleBonuses: pbs,
        },
        {
            header: "Best (+GLOBALIST)",
            bonus: "GLOBALIST",
            solutions: solGlobalists,
            possibleBonuses: pbsGlobalists,
        },
        {
            header: "Best (+SUPERFLEX)",
            bonus: "SUPERFLEX",
            solutions: solSuperflexes,
            possibleBonuses: pbsSuperflexes,
        }
    ];

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
                <ProblemList model={model} problems={ps} hiddenProblems={hiddenProblems} greenBackgroundProblems={greenBackgroundProblems} columns={columns} showViewer={!formFilter.hideViewer} />
            </FormGroup>
        </Container>
    );
};

export interface FrontPageProps {
    model: Model;
}

export const FrontPage = (props: FrontPageProps) => {
    const {model} = props;

    return (
        <FrontPageProblemList model={model} />
    );
};
