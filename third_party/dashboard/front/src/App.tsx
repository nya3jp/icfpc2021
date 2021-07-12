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
                            <Button variant="outlined" color="inherit" href="https://docs.google.com/document/d/1AriiBG9OYnb0XMxa0ogc3W_LWlh2FJObXF3rT1uTfVk/edit#">共有メモ</Button>
                            <Button variant="outlined" color="inherit" href="https://nya3jp.github.io/icfpc2021/fcc7938b3c545e6ff51b101ea86f548b/">手動UI</Button>
                            <Button variant="outlined" color="inherit" href="https://poses.live/problems">ICFPC問題ページ</Button>
                            <Button variant="outlined" color="inherit" href="https://poses.live/teams">チームスコア</Button>
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
                        <Route path="/tasks/:taskID">
                            <TaskPage model={model} />
                        </Route>
                        <Route path="/tags/:tag">
                            <TagPage model={model} />
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
