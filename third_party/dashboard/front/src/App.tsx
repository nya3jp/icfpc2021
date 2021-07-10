import React, {useState} from 'react';
import {BrowserRouter as Router, Link, Route, Switch} from 'react-router-dom';
import {PoseMap, SolutionMap} from './types';
import {FrontPage} from './FrontPage';
import {SolutionPage} from './SolutionPage';

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

  return (
    <Router>
      <nav>
        <ul>
          <li>
            <Link to="/">Home</Link>
          </li>
        </ul>
      </nav>
      <Switch>
        <Route path="/problems/:problemID/solutions/:solutionID">
          <SolutionPage solutions={solutions} ensureSolution={ensureSolution} poses={poses} ensurePoses={ensurePose} />
        </Route>
        <Route path="/">
          <FrontPage solutions={solutions} ensureSolution={ensureSolution} />
        </Route>
      </Switch>
    </Router>
  );
}
