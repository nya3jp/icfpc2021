import React, {useEffect, useState} from 'react';
import {
    BrowserRouter as Router,
    Link,
    Route,
    Switch,
    useParams
} from 'react-router-dom';
import List from '@material-ui/core/List';
import ListItem from '@material-ui/core/ListItem';
import ListItemText from '@material-ui/core/ListItemText';
import Typography from '@material-ui/core/Typography';
import {Pose, Solution} from './types';

const solutionKey = (problem_id: string, solution_id: string): string => problem_id + "-" + solution_id;
type SolutionMap = {[key: string]: Solution};
type PoseMap = {[key: string]: Pose};

interface RecentSolutionsListProps {
    solutions: SolutionMap;
    recentSolutions: Solution[];
}

const RecentSolutionsList = (props: RecentSolutionsListProps) => {
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

interface HomeProps {
    solutions: SolutionMap;
    ensureSolution: (problemID: string, solutionID: string) => void;
}

interface HomeState {
    loading: boolean;
    recentSolutions: Solution[];
}

const Home = (props: HomeProps) => {
  const {solutions, ensureSolution} = props;
  const [appState, setAppState] = useState<HomeState>({
    loading: false,
    recentSolutions: [],
  });

  useEffect(() => {
    setAppState({loading: true, recentSolutions: []});
      fetch(`https://spweek.badalloc.com/api/solutions`)
      .then((res) => res.json())
      .then((recentSolutions: Solution[]) => {
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

interface SolutionDumpProps {
    solutions: SolutionMap;
    ensureSolution: (problemID: string, solutionID: string) => void;
    poses: PoseMap;
    ensurePoses: (problemID: string, solutionID: string) => void;
}

const SolutionDump = (props: SolutionDumpProps) => {
  const {solutions, ensureSolution, poses, ensurePoses} = props;
  const {problemID, solutionID} = useParams<{problemID: string, solutionID: string}>();

  useEffect(() => {
    ensureSolution(problemID, solutionID);
    ensurePoses(problemID, solutionID);
  })

  const key = problemID + "-" + solutionID;
  console.log(poses);
  if (poses[key]) {
    return <div>{JSON.stringify(poses[key])}</div>
  }
  return (
    <div></div>
  );
};

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
          <SolutionDump solutions={solutions} ensureSolution={ensureSolution} poses={poses} ensurePoses={ensurePose} />
        </Route>
        <Route path="/">
          <Home solutions={solutions} ensureSolution={ensureSolution} />
        </Route>
      </Switch>
    </Router>
  );
}
