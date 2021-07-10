import {Problem} from './types';

export class ProblemSet {
    constructor() {
    }

    public async getProblemIds(): Promise<number[]> {
        return Array.from(Array(88).keys()).map(num => num + 1);
    }

    public async getProblem(id: number): Promise<Problem> {
        const res = await fetch(`./problems/${id}.problem`);
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json();
    }
}
