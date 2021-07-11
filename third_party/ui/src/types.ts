export type Point = [number, number];

export type Edge = [number, number];

export type Hole = Point[];

export type Pose = Point[];

export interface Figure {
    edges: Edge[]
    vertices: Pose
}

export interface Bonus {
    bonus: string
    position: Point
    problem: number
}

export interface Problem {
    hole: Hole
    figure: Figure
    epsilon: number
    bonuses: Bonus[]
}

export interface Solution {
    id: number
    pose: Pose
}
