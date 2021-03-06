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

import {useState} from 'react';
import {HashRouter as Router, Link, Route, Switch} from 'react-router-dom';
import {FrontPage} from './FrontPage';
import {SolutionPage} from './SolutionPage';

import {makeStyles} from '@material-ui/core/styles';
import AppBar from '@material-ui/core/AppBar';
import Box from '@material-ui/core/Box';
import Toolbar from '@material-ui/core/Toolbar';
import Button from '@material-ui/core/Button';
import Typography from '@material-ui/core/Typography';
import {Model} from './model';
import {ProblemPage} from './ProblemPage';
import {SolutionSetPage} from './SolutionSetPage';
import {TaskPage} from './TaskPage';
import {TagPage} from './TagPage';
import {TagComparePage} from './TagComparePage';
import {TasksPage} from './TasksPage';
import {SubmitPage} from './SubmitPage';

const useStyles = makeStyles((theme) => ({
    root: {
        flexGrow: 1,
    },
    title: {
        flexGrow: 1,
    },
    buttons: {
        '& > *': {
            margin: theme.spacing(1),
        },
    },
}));

export default function App() {
    const [model] = useState<Model>(() => new Model());
    const classes = useStyles();

    return (
        <Router>
            <div className={classes.root}>
                <AppBar position="static">
                    <Toolbar>
                        <Typography variant="h6" className={classes.title}>
                        </Typography>
                        <div className={classes.buttons}>
                            <Button variant="outlined" color="inherit" href="https://docs.google.com/document/d/1AriiBG9OYnb0XMxa0ogc3W_LWlh2FJObXF3rT1uTfVk/edit#">????????????</Button>
                            <Button variant="outlined" color="inherit" href="https://nya3jp.github.io/icfpc2021/fcc7938b3c545e6ff51b101ea86f548b/">??????UI</Button>
                            <Button variant="outlined" color="inherit" href="https://poses.live/problems">ICFPC???????????????</Button>
                            <Button variant="outlined" color="inherit" href="https://poses.live/teams">??????????????????</Button>
                            <Button variant="outlined" color="inherit" component={Link} to="/tasks">???????????????</Button>
                            <Button variant="outlined" color="inherit" component={Link} to="/">Home</Button>
                        </div>
                    </Toolbar>
                </AppBar>
                <Box pt={1}>
                    <Switch>
                        <Route path="/solutionsets/">
                            <SolutionSetPage model={model} />
                        </Route>
                        <Route path="/solutions/:solutionID">
                            <SolutionPage model={model} />
                        </Route>
                        <Route path="/problems/:problemID">
                            <ProblemPage model={model} />
                        </Route>
                        <Route exact path="/tasks/">
                            <TasksPage model={model} />
                        </Route>
                        <Route path="/tasks/:taskID">
                            <TaskPage model={model} />
                        </Route>
                        <Route path="/submit/:problemID">
                            <SubmitPage model={model} />
                        </Route>
                        <Route path="/tags/:tag">
                            <TagPage model={model} />
                        </Route>
                        <Route path="/compare/">
                            <TagComparePage model={model} />
                        </Route>
                        <Route path="/">
                            <FrontPage model={model} />
                        </Route>
                    </Switch>
                </Box>
            </div>
        </Router>
    );
}
