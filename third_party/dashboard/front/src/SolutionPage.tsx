import {useParams} from 'react-router-dom';
import {useEffect, useState} from 'react';
import {Solution} from './types';
import {Link} from 'react-router-dom';

import Chip from '@material-ui/core/Chip';
import Paper from '@material-ui/core/Paper';
import Table from '@material-ui/core/Table';
import TableBody from '@material-ui/core/TableBody';
import TableCell from '@material-ui/core/TableCell';
import TableContainer from '@material-ui/core/TableContainer';
import TableHead from '@material-ui/core/TableHead';
import TableRow from '@material-ui/core/TableRow';
import {Model} from './model';

export interface SolutionPageProps {
    model: Model;
}

export const SolutionPage = (props: SolutionPageProps) => {
    const {model} = props;
    const {solutionID} = useParams<{ solutionID: string }>();
    const [solution, setSolution] = useState<Solution | null>(null);

    useEffect(() => {
        (async () => {
            setSolution(await model.getSolution(+solutionID));
        })();
    });

    if (!solution) {
      return <div></div>
    }
    const createdAt = new Date();
    createdAt.setTime(solution.created_at * 1000);
    return (
      <div>
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
                <TableCell component="th" scope="row" align="right">ProblemID</TableCell>
                <TableCell>{solution.problem_id}</TableCell>
              </TableRow>
              <TableRow>
                <TableCell component="th" scope="row" align="right">SolutionID</TableCell>
                <TableCell>{solution.solution_id}</TableCell>
              </TableRow>
              <TableRow>
                <TableCell component="th" scope="row" align="right">Created at</TableCell>
                <TableCell>{createdAt.toString()}</TableCell>
              </TableRow>
              {solution.tags &&
                <TableRow>
                  <TableCell component="th" scope="row" align="right">Tags</TableCell>
                  <TableCell>{solution.tags.map((tag) => <Link to={`/tags/${tag}`}><Chip label={tag} /></Link>)}</TableCell>
                </TableRow>
              }
            </TableBody>
          </Table>
        </TableContainer>
        <pre>{JSON.stringify(solution)}</pre>
      </div>
    );
};
