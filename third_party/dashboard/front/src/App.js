import React, {useEffect, useState} from 'react';
import {
  BrowserRouter as Router,
  Switch,
  Route,
  Link,
  useParams
} from 'react-router-dom';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import ListItemText from '@material-ui/core/ListItemText';
import Typography from '@material-ui/core/Typography';

const solutionKey = (problem_id, solution_id) => problem_id + "-" + solution_id;

const RecentSolutionsList = (props) => {
  const {solutions, recentSolutions} = props;
  if (!recentSolutions || recentSolutions.length === 0) return <p>No solutions</p>;
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
            return <Link key={key} to={link}><ListItem><ListItemText primary={primary} /></ListItem></Link>;
          }
          return <Link key={key} to={link}><ListItem><ListItemText>Problem {problem_id} Solution {solution_id}</ListItemText></ListItem></Link>;
        })}
      </List>
    </div>
  );
};

const Home = (props) => {
  const {solutions, ensureSolution} = props;
  const [appState, setAppState] = useState({
    loading: false,
    recentSolutions: null,
  });

  useEffect(() => {
    setAppState({loading: true});
    fetch(`https://spweek.badalloc.com/api/solutions`)
      .then((res) => res.json())
      .then((recentSolutions) => {
        setAppState({loading: false, recentSolutions: recentSolutions});
        recentSolutions.map((solution) => {
          ensureSolution(solution.problem_id, solution.solution_id);
        });
      });
  }, [setAppState]);

  return (
    <div>
      <h2></h2>
      <RecentSolutionsList solutions={solutions} recentSolutions={appState.recentSolutions} />
    </div>
  );
};

const Solution = (props) => {
  const {solutions, ensureSolution, solutionFiles, ensureSolutionFile} = props;
  const {problemID, solutionID} = useParams();

  useEffect(() => {
    ensureSolution(problemID, solutionID);
    ensureSolutionFile(problemID, solutionID);
  })

  const key = problemID + "-" + solutionID;
  console.log(solutionFiles);
  if (solutionFiles[key]) {
    return <div>{JSON.stringify(solutionFiles[key])}</div>
  }
  return (
    <div></div>
  );
};

export default function App() {
  const [solutions, setSolutions] = useState({});
  const [solutionFiles, setSolutionFiles] = useState({});

  const ensureSolution = (problemID, solutionID) => {
    const idx = problemID + "-" + solutionID;
    if (solutions[idx]) {
      return;
    }
    fetch(`https://spweek.badalloc.com/api/problems/` + problemID + `/solutions/` + solutionID + `/meta`)
      .then((res) => res.json())
      .then((s) => {
        var obj = {};
        obj[idx] = s;
        setSolutions(obj);
      });
  };
  const ensureSolutionFile = (problemID, solutionID) => {
    const idx = problemID + "-" + solutionID;
    if (solutionFiles[idx]) {
      return;
    }
    fetch(`https://spweek.badalloc.com/api/problems/` + problemID + `/solutions/` + solutionID)
      .then((res) => res.json())
      .then((s) => {
        var obj = {};
        obj[idx] = s;
        setSolutionFiles(obj);
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
          <Solution solutions={solutions} ensureSolution={ensureSolution} solutionFiles={solutionFiles} ensureSolutionFile={ensureSolutionFile} />
        </Route>
        <Route path="/">
          <Home solutions={solutions} ensureSolution={ensureSolution} />
        </Route>
      </Switch>
    </Router>
  );
}
