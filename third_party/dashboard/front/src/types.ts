export type Point = [number, number];
export type Edge = [number, number];
export type Hole = Point[];

export interface Figure {
    edges: Edge[];
    vertices: Point[];
}

export interface ProblemData {
    hole: Hole;
    figure: Figure;
    epsilon: number;
    bonuses: Bonus[];
}

export interface Problem {
    problem_id: number;
    created_at: number;
    minimal_dislike: number;
    data: ProblemData;
}

export interface SolutionData {
    vertices: Point[];
    bonuses: UsedBonus[];
}

export interface Solution {
    problem_id: number;
    solution_id: number;
    created_at: number;
    tags: string[];
    dislike: number;
    reject_reason: string;
    acquired_bonuses: UsedBonus[];
    data: SolutionData;
}

export interface Bonus {
    bonus: string;
    position: Point;
    problem: number;
}

export interface UsedBonus {
    bonus: string;
    problem: number;
}

export interface GotBonus {
    from_problem_id: number;
    kind: string;
};

export type BonusMap = {[key: number]: GotBonus[]};

export interface TaskStatus {
    task: Task;
    state: 'PENDING' | 'RUNNING' | 'FINISHED';
    worker?: string;
    result?: TaskResult;
}

export interface Task {
    id: TaskId;
    spec: TaskSpec;
}

export interface TaskId {
    id: number;
}

export interface TaskSpec {
    command: TaskCommand;
    packages?: TaskPackage[];
    constraints?: TaskConstraints;
    limits?: TaskLimits;
}

export interface TaskCommand {
    shell: string;
}

export interface TaskPackage {
    url: string
}

export interface TaskConstraints {
    priority: number;
}

export interface TaskLimits {
    time?: string;
}

export interface TaskResult {
    exitCode?: number;
    error?: string;
}
