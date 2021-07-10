import React, {useEffect, useState} from 'react';
import {Solution, solutionKey, SolutionMap} from './types';
import Typography from '@material-ui/core/Typography';
import List from '@material-ui/core/List';
import {Link} from 'react-router-dom';
import ListItem from '@material-ui/core/ListItem';
import ListItemText from '@material-ui/core/ListItemText';

interface RecentSolutionsListProps {
    solutions: SolutionMap;
    recentSolutions: Solution[];
}

const RecentSolutionsList = (props: RecentSolutionsListProps) => {
    const {solutions, recentSolutions} = props;
    if (!recentSolutions || recentSolutions.length === 0) return <p>No
        solutions</p>;
    return (
        <div>
            <Typography variant="h6">
                Recent solutions
            </Typography>
            <List>
                {recentSolutions.map(({problem_id, solution_id}) => {
                    const key = solutionKey(problem_id, solution_id);
                    const link = `/problems/${problem_id}/solutions/${solution_id}`;
                    if (solutions[key]) {
                        const s = solutions[key];
                        const primary = `Problem ${problem_id} Solution ${solution_id} ${s.tags} ${s.solution_sets}`;
                        return <Link key={key} to={link}><ListItem><ListItemText
                            primary={primary}/></ListItem></Link>;
                    }
                    return <Link key={key}
                                 to={link}><ListItem><ListItemText>Problem {problem_id} Solution {solution_id}</ListItemText></ListItem></Link>;
                })}
            </List>
        </div>
    );
};

export interface FrontPageProps {
    solutions: SolutionMap;
    ensureSolution: (problemID: string, solutionID: string) => void;
}

interface FrontPageState {
    loading: boolean;
    recentSolutions: Solution[];
}

export const FrontPage = (props: FrontPageProps) => {
    const {solutions, ensureSolution} = props;
    const [appState, setAppState] = useState<FrontPageState>({
        loading: false,
        recentSolutions: [],
    });

    useEffect(() => {
        setAppState({loading: true, recentSolutions: []});
        fetch(`https://spweek.badalloc.com/api/solutions`)
            .then((res) => res.json())
            .then((recentSolutions: Solution[]) => {
                setAppState({loading: false, recentSolutions: recentSolutions});
                recentSolutions.map((solution) => {
                    ensureSolution(solution.problem_id, solution.solution_id);
                });
            });
    }, [setAppState]);

    return (
        <div>
            <h2></h2>
            <RecentSolutionsList solutions={solutions}
                                 recentSolutions={appState.recentSolutions}/>
        </div>
    );
};
