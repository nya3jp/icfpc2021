import {useHistory, useLocation, useParams} from 'react-router-dom';
import {ChangeEvent, useEffect, useState} from 'react';
import {Problem, TaskSpec, TaskStatus} from './types';

import {makeStyles} from '@material-ui/core/styles';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import Container from '@material-ui/core/Container';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import {Model} from './model';
import {Button, MenuItem, TextField, Typography} from '@material-ui/core';
import {SolutionsTable} from './SolutionsTable';
import {Viewer} from './editor/Viewer';

function useQuery(): URLSearchParams {
    return new URLSearchParams(useLocation().search);
}

const useStyles = makeStyles((theme) => ({
    root: {
        '& > *': {
            margin: theme.spacing(1),
        },
    },
}));

export interface SubmitPageProps {
    model: Model;
}

interface SolverSpec {
    name: string;
    defaultArgs: string;
}

export const SubmitPage = (props: SubmitPageProps) => {
    const {model} = props;

    const solvers: SolverSpec[] = [
        {name: 'tanakh-solver.bb6b8fc', defaultArgs: '--time-limit=1200'},
        {name: 'chun-tachyon-solver.487245d', defaultArgs: '--time-limit=300'},
    ];

    const classes = useStyles();
    const history = useHistory();
    const problemId = parseInt(useParams<{ problemID: string }>().problemID);
    const [problem, setProblem] = useState<Problem | null>(null);
    const [solver, setSolver] = useState<string>(solvers[0].name);
    const [extraArgs, setExtraArgs] = useState<string>(solvers[0].defaultArgs);

    useEffect(() => {
        (async () => {
            const problem = await model.getProblem(problemId);
            setProblem(problem);
        })();
    }, [problemId]);

    if (problem === null) {
        return <div>Loading...</div>;
    }

    const baseArgs = `exec ./${solver.split(/\./)[0]} solve ${problemId}`;
    const handleSolverChange = (ev: ChangeEvent<HTMLInputElement>) => {
        for (const spec of solvers) {
            if (ev.target.value === spec.name) {
                setSolver(spec.name);
                setExtraArgs(spec.defaultArgs);
            }
        }
    };
    const handleExtraArgsChange = (ev: ChangeEvent<HTMLInputElement>) => {
        setExtraArgs(ev.target.value);
    }
    const handleSubmit = () => {
        const spec: TaskSpec = {
            command: {
                shell: `${baseArgs} ${extraArgs}`,
            },
            constraints: {
                priority: 1000,
            },
            limits: {
                time: '30m',
            },
            packages: [{
                url: `https://storage.googleapis.com/special-weekend-2021-flex/packages/${solver}.tar.gz`,
            }]
        };
        (async () => {
            const taskId = await model.addTask(spec);
            history.push(`/tasks/${taskId}`);
        })();
    };

    return (
        <Container>
            <Typography variant={'h3'}>Schedule Solver for Problem {problem.problem_id}</Typography>
            <div>
                <Viewer problem={problem} />
            </div>

            <form className={classes.root} noValidate autoComplete="off">
                <TextField
                    select
                    label="Solver"
                    value={solver}
                    onChange={handleSolverChange}
                    variant="outlined"
                >
                    {solvers.map((spec) => (
                        <MenuItem key={spec.name} value={spec.name}>
                            {spec.name}
                        </MenuItem>
                    ))}
                </TextField>
                <TextField
                    fullWidth
                    label="Base arguments"
                    value={baseArgs}
                    variant="outlined"
                    disabled
                />
                <TextField
                    fullWidth
                    label="Extra arguments"
                    value={extraArgs}
                    onChange={handleExtraArgsChange}
                    variant="outlined"
                />
                <Button variant="contained" color="primary" onClick={handleSubmit}>
                    Submit
                </Button>
            </form>
        </Container>
    );
};
