import {Problem, Solution} from './types';

export function pointRatio(problem: Problem, solution: Solution): number {
    return Math.sqrt((problem.minimal_dislike + 1) / (solution.dislike + 1));
};

export function maxScore(problem: Problem): number {
    return Math.ceil(1000.0
        * Math.log2((problem.data.figure.vertices.length
            * problem.data.figure.edges.length
            * problem.data.hole.length) / 6.0));
};

export interface ScoreInfo {
    maxScore: number;
    score: number;
    ratio: number;
};

export function scoreInfo(problem: Problem, solution: Solution): ScoreInfo {
    let ratio = pointRatio(problem, solution);
    let ms = maxScore(problem);
    let score = Math.ceil(ms * ratio);
    return {
        maxScore: ms,
        score: score,
        ratio: ratio,
    };
}
