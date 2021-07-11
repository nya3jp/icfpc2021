import {Problem, Solution} from './types';

const PROD_BASE_URL = 'https://spweek.badalloc.com';
const DEV_BASE_URL = 'http://localhost:8080'

// Client is the API client for our dashboard server.
class Client {
    private readonly baseURL: string;

    constructor() {
        this.baseURL = (window.location.hostname === 'localhost' ? PROD_BASE_URL : PROD_BASE_URL);
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

    async submitSolution(id: number): Promise<string> {
        const res = await fetch(`${this.baseURL}/api/solutions/${id}/submit`, {method: 'POST'});
        return await res.text();
    }
}

// Model wraps Client for caching.
export class Model {
    private readonly problems = new Map<number, Promise<Problem>>();
    private readonly solutions = new Map<number, Promise<Solution>>();

    constructor(private readonly client = new Client()) {}

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

    submitSolution(id: number): Promise<string> {
        return this.client.submitSolution(id);
    }
}
