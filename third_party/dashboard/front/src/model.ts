import {Problem, Solution, TaskSpec, TaskStatus} from './types';

const PROD_BASE_URL = 'https://spweek.badalloc.com';
const DEV_BASE_URL = 'http://localhost:8080'

// Client is the API client for our dashboard server.
class Client {
    private readonly baseURL: string;

    constructor() {
        this.baseURL = (window.location.hostname === 'localhost' ? DEV_BASE_URL : PROD_BASE_URL);
    }

    async getProblems(): Promise<Problem[]> {
        const res = await fetch(`${this.baseURL}/api/problems`);
        return await res.json();
    }

    async getProblem(id: number): Promise<Problem> {
        const res = await fetch(`${this.baseURL}/api/problems/${id}`);
        return await res.json();
    }

    async getSolution(id: number): Promise<Solution> {
        const res = await fetch(`${this.baseURL}/api/solutions/${id}`);
        return await res.json();
    }

    async getSolutionsForProblem(id: number): Promise<Solution[]> {
        const res = await fetch(`${this.baseURL}/api/problems/${id}/solutions`);
        return await res.json();
    }

    async getSolutionsForTag(tag: string): Promise<Solution[]> {
        const res = await fetch(`${this.baseURL}/api/solutions?tag=${tag}`);
        return await res.json();
    }

    async submitSolution(id: number): Promise<string> {
        const res = await fetch(`${this.baseURL}/api/solutions/${id}/submit`, {method: 'POST'});
        return await res.text();
    }

    async triggerSolver(problemID: number, bonus: string, penaltyRatio:number, timeLimitSec: number, deadlineSec: number): Promise<number> {
        const res = await fetch(`${this.baseURL}/api/problems/${problemID}/solve?penalty_ratio=${penaltyRatio}&deadline=${deadlineSec}&time_limit=${timeLimitSec}&bonus=${bonus}`, {method: 'POST'});
        return await res.json();
    }

    async getTaskStatus(id: number): Promise<TaskStatus> {
        const res = await fetch(`${this.baseURL}/api/tasks/info/${id}`);
        return await res.json();
    }

    async getAllTaskStatuses(): Promise<TaskStatus[]> {
        const res = await fetch(`${this.baseURL}/api/tasks/all`);
        return await res.json();
    }

    async setSolutionTag(id: number, tag: string): Promise<Solution> {
        const res = await fetch(`${this.baseURL}/api/solutions/${id}/tags?tag=${tag}`, {method: 'POST'});
        return await res.json();
    }

    async deleteSolutionTag(id: number, tag: string): Promise<void> {
        const res = await fetch(`${this.baseURL}/api/solutions/${id}/tags?tag=${tag}`, {method: 'DELETE'});
        return await res.json();
    }

    async addTask(spec: TaskSpec): Promise<number> {
        const init: RequestInit = {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify(spec),
        };
        const res = await fetch(`${this.baseURL}/api/tasks/addjson`, init);
        return await res.json();
    }
}

// Model wraps Client for caching.
export class Model {
    private readonly problems = new Map<number, Promise<Problem>>();
    private readonly solutions = new Map<number, Promise<Solution>>();

    constructor(private readonly client = new Client()) {
    }

    getProblems(): Promise<Problem[]> {
        // Problems change over time, do not cache.
        return this.client.getProblems();
    }

    getProblem(id: number): Promise<Problem> {
        const cached = this.problems.get(id);
        if (cached) {
            return cached;
        }
        const fresh = this.client.getProblem(id);
        this.problems.set(id, fresh);
        return fresh;
    }

    getSolution(id: number): Promise<Solution> {
        const cached = this.solutions.get(id);
        if (cached) {
            return cached;
        }
        const fresh = this.client.getSolution(id);
        this.solutions.set(id, fresh);
        return fresh;
    }

    getSolutionsForProblem(id: number): Promise<Solution[]> {
        // Solutions for a problem change over time, do not cache.
        return this.client.getSolutionsForProblem(id);
    }

    getSolutionsForTag(tag: string): Promise<Solution[]> {
        return this.client.getSolutionsForTag(tag);
    }

    submitSolution(id: number): Promise<string> {
        return this.client.submitSolution(id);
    }

    triggerSolver(problemID: number, bonus: string, penaltyRatio: number, timeLimitSec: number, deadlineSec: number): Promise<number> {
        return this.client.triggerSolver(problemID, bonus, penaltyRatio, timeLimitSec, deadlineSec);
    }

    getTaskStatus(id: number): Promise<TaskStatus> {
        return this.client.getTaskStatus(id);
    }

    getAllTaskStatuses(): Promise<TaskStatus[]> {
        return this.client.getAllTaskStatuses();
    }

    setSolutionTag(id: number, tag: string): Promise<Solution> {
        const fresh = this.client.setSolutionTag(id, tag);
        this.solutions.set(id, fresh);
        return fresh;
    }

    deleteSolutionTag(id: number, tag: string): Promise<void> {
        const ret = this.client.deleteSolutionTag(id, tag);
        this.solutions.delete(id);
        return ret;
    }

    addTask(spec: TaskSpec): Promise<number> {
        return this.client.addTask(spec)
    }
}
