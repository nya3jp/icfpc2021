import React, {useState} from 'react';
import {BrowserRouter as Router, Link, Route, Switch} from 'react-router-dom';
import {PoseMap, SolutionMap} from './types';
import {FrontPage} from './FrontPage';
import {SolutionPage} from './SolutionPage';

import {makeStyles} from '@material-ui/core/styles';
import AppBar from '@material-ui/core/AppBar';
import Toolbar from '@material-ui/core/Toolbar';
import Container from '@material-ui/core/Container';
import Button from '@material-ui/core/Button';
import Typography from '@material-ui/core/Typography';

const useStyles = makeStyles((theme) => ({
  root: {
    flexGrow: 1,
  },
  title: {
    flexGrow: 1,
  },
}));


export default function App() {
  const [solutions, setSolutions] = useState<SolutionMap>({});
  const [poses, setPoses] = useState<PoseMap>({});

  const ensureSolution = (problemID: string, solutionID: string) => {
    const idx = problemID + "-" + solutionID;
    if (solutions[idx]) {
      return;
    }
    fetch(`https://spweek.badalloc.com/api/problems/` + problemID + `/solutions/` + solutionID + `/meta`)
      .then((res) => res.json())
      .then((s) => {
        const obj: SolutionMap = {};
        obj[idx] = s;
        setSolutions(obj);
      });
  };
  const ensurePose = (problemID: string, solutionID: string) => {
    const idx = problemID + "-" + solutionID;
    if (poses[idx]) {
      return;
    }
    fetch(`https://spweek.badalloc.com/api/problems/` + problemID + `/solutions/` + solutionID)
      .then((res) => res.json())
      .then((s) => {
        const obj: PoseMap = {};
        obj[idx] = s;
        setPoses(obj);
      });
  };

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
            <Route path="/problems/:problemID/solutions/:solutionID">
              <SolutionPage solutions={solutions} ensureSolution={ensureSolution} poses={poses} ensurePoses={ensurePose} />
            </Route>
            <Route path="/">
              <FrontPage solutions={solutions} ensureSolution={ensureSolution} />
            </Route>
          </Switch>
        </Container>
      </Router>
    </div>
  );
}
