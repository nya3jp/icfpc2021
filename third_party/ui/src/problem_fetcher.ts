import { Problem } from './types';

export function fetch_problem(problem_id: number): Promise<Problem> {
    return fetch('./problems/' + problem_id.toString() + ".problem")
        .then(res => {
            if (!res.ok) {
                throw new Error(res.statusText)
            }
            return res.json() as Promise<Problem>;
        })
}

export function problem_list(): number[] {
    return Array.from(Array(88).keys()).map(num => num + 1)
}