import {useParams} from 'react-router-dom';
import React, {useEffect} from 'react';
import {PoseMap, SolutionMap} from './types';

export interface SolutionPageProps {
    solutions: SolutionMap;
    ensureSolution: (problemID: string, solutionID: string) => void;
    poses: PoseMap;
    ensurePoses: (problemID: string, solutionID: string) => void;
}

export const SolutionPage = (props: SolutionPageProps) => {
    const {solutions, ensureSolution, poses, ensurePoses} = props;
    const {
        problemID,
        solutionID
    } = useParams<{ problemID: string, solutionID: string }>();

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
