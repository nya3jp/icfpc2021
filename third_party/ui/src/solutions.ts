import {Solution, Pose} from './types';

export class SolutionSet {
    constructor() {
    }

    public async getSolution(id: number): Promise<Solution> {
        const res = await fetch(`https://spweek.badalloc.com/api/solutions/${id}`);
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json().then((m: any) => {
            return {
                id: m.solution_id,
                pose: m.data.vertices,
            } as Solution;
        });
    }

    public async addSolution(problemId: number, pose: Pose): Promise<number> {
        const res = await fetch(`https://spweek.badalloc.com/api/problems/${problemId}/solutions`, {
            method: "post",
            body: JSON.stringify({vertices: pose}),
        });
        if (!res.ok) {
            throw new Error(res.statusText);
        }
        return res.json().then((m: any) => {
            return m.solution_id;
        });
    }
}
