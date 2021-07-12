import {useLocation, useParams} from 'react-router-dom';
import {useEffect, useState} from 'react';
import {Problem, Solution, TaskStatus} from './types';
import {Link} from 'react-router-dom';

import {makeStyles} from '@material-ui/core/styles';
import Button from '@material-ui/core/Button';
import Chip from '@material-ui/core/Chip';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import Container from '@material-ui/core/Container';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import {Model} from './model';
import {Typography} from '@material-ui/core';

import {Viewer} from './editor/Viewer';
import {scoreInfo} from './utils';
import {EditButton, OfficialSubmitButton} from './buttons';

const useStyles = makeStyles((theme) => ({
    buttons: {
        '& > *': {
            margin: theme.spacing(1),
        },
    },
}));

export interface TaskPageProps {
    model: Model;
}

export const TaskPage = (props: TaskPageProps) => {
    const {model} = props;

    const classes = useStyles();
    const taskID = parseInt(useParams<{taskID: string}>().taskID);
    const [taskStatus, setTaskStatus] = useState<TaskStatus | null>(null);
    const [stdout, setStdout] = useState<string>('');
    const [stderr, setStderr] = useState<string>('');

    useEffect(() => {
        (async () => {
            const taskStatus = await model.getTaskStatus(taskID);
            if (taskStatus.state === 'FINISHED') {
                setStdout(await (await fetch(`https://storage.googleapis.com/special-weekend-2021-flex/prod/tasks/${taskID}/stdout.txt`)).text());
                setStderr(await (await fetch(`https://storage.googleapis.com/special-weekend-2021-flex/prod/tasks/${taskID}/stderr.txt`)).text());
            }
            setTaskStatus(taskStatus);
        })();
    }, [taskID]);

    if (!taskStatus) {
        return <div>Loading...</div>;
    }

    return (
        <Container>
            <Typography variant={'h3'}>Task {taskID}</Typography>
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
                            <TableCell component="th" scope="row" align="right">Command</TableCell>
                            <TableCell>{taskStatus.task.spec.command.shell}</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Packages</TableCell>
                            <TableCell>
                                {
                                    taskStatus.task.spec.packages?.map((pkg) => <div><a href={pkg.url}>{pkg.url}</a></div>)
                                }
                            </TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">State</TableCell>
                            <TableCell>{taskStatus.state}</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Assigned Worker</TableCell>
                            <TableCell>{taskStatus.worker}</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Exit Code</TableCell>
                            <TableCell>
                                {
                                    taskStatus.result?.error || taskStatus.result?.exitCode
                                }
                            </TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Standard Output</TableCell>
                            <TableCell><pre style={{whiteSpace: 'pre-wrap'}}>{stdout}</pre></TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell component="th" scope="row" align="right">Standard Error</TableCell>
                            <TableCell><pre style={{whiteSpace: 'pre-wrap'}}>{stderr}</pre></TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </TableContainer>
        </Container>
    );
};
