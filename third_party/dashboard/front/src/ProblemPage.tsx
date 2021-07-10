import {useParams} from 'react-router-dom';
import React, {useEffect, useState} from 'react';
import {Problem} from './types';
import {Link} from 'react-router-dom';
import {Model} from './model';

export interface ProblemPageProps {
    model: Model;
}

export const ProblemPage = (props: ProblemPageProps) => {
    const {model} = props;
    const {problemID} = useParams<{ problemID: string }>();
    const [problem, setProblem] = useState<Problem | null>(null);

    useEffect(() => {
        (async () => {
            setProblem(await model.getProblem(problemID));
        })();
    });

    if (!problem) {
      return <div></div>;
    }

    const createdAt = new Date();
    createdAt.setTime(problem.created_at * 1000);
    return (
      <div>
        <pre>{JSON.stringify(problem)}</pre>
      </div>
    );
};
