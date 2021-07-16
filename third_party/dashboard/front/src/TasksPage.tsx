/**
 * Copyright 2021 Team Special Weekend
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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
