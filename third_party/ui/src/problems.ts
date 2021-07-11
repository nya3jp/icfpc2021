import {Problem} from './types';

const MAX_PROBLEM_ID: number = 132;

export class ProblemSet {
    constructor() {
    }

    public async getProblemIds(): Promise<number[]> {
        return Array.from(Array(MAX_PROBLEM_ID).keys()).map(num => num + 1);
    }

    public async getProblem(id: number): Promise<Problem> {
        const res = await fetch(`./problems/${id}.problem`);
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json();
    }
}
