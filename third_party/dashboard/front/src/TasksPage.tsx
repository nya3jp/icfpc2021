import {useEffect, useState} from 'react';
import {Link} from 'react-router-dom';
import {TaskStatus} from './types';

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

export interface TasksPageProps {
    model: Model;
}

export const TasksPage = (props: TasksPageProps) => {
    const {model} = props;

    const [tasks, setTasks] = useState<TaskStatus[] | null>(null);

    useEffect(() => {
        (async () => {
            setTasks(await model.getAllTaskStatuses());
        })();
    }, []);

    if (tasks === null) {
        return <div>Loading...</div>;
    }

    const taskRows = tasks.map((task) => (
        <TableRow>
            <TableCell component="th" scope="row" align="right"><Link to={`/tasks/${task.task.id.id}`}>{task.task.id.id}</Link></TableCell>
            <TableCell>{task.state}</TableCell>
            <TableCell><span style={{whiteSpace: 'pre'}}>{task.task.spec.command.shell}</span></TableCell>
        </TableRow>
    ));
    return (
        <Container>
            <Typography variant={'h3'}>Tasks</Typography>
            <TableContainer component={Paper}>
                <Table size="small" aria-label="a dense table">
                    <TableHead>
                        <TableRow>
                            <TableCell align="right">ID</TableCell>
                            <TableCell>State</TableCell>
                            <TableCell>Command</TableCell>
                        </TableRow>
                    </TableHead>
                    <TableBody>
                        { taskRows }
                    </TableBody>
                </Table>
            </TableContainer>
        </Container>
    );
};
