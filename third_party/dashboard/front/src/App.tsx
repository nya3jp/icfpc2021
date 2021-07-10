import React, {useState} from 'react';
import {BrowserRouter as Router, Link, Route, Switch} from 'react-router-dom';
import {FrontPage} from './FrontPage';
import {SolutionPage} from './SolutionPage';

import {makeStyles} from '@material-ui/core/styles';
import AppBar from '@material-ui/core/AppBar';
import Toolbar from '@material-ui/core/Toolbar';
import Container from '@material-ui/core/Container';
import Button from '@material-ui/core/Button';
import Typography from '@material-ui/core/Typography';
import {Model} from './model';

const useStyles = makeStyles((theme) => ({
  root: {
    flexGrow: 1,
  },
  title: {
    flexGrow: 1,
  },
}));

export default function App() {
  const [model, _] = useState<Model>(() => new Model());
  const classes = useStyles();

  return (
    <div className={classes.root}>
      <Router>
        <AppBar position="static">
          <Toolbar>
            <Typography variant="h6" className={classes.title}>
            </Typography>
            <Button color="inherit" href="https://poses.live/problems">ICFPC Dash</Button>
            <Button color="inherit" component={Link} to="/">Home</Button>
          </Toolbar>
        </AppBar>
        <Container>
          <Switch>
            <Route path="/solutions/:solutionID">
              <SolutionPage model={model} />
            </Route>
            <Route path="/">
              <FrontPage model={model} />
            </Route>
          </Switch>
        </Container>
      </Router>
    </div>
  );
}
