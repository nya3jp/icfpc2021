/**
 * Copyright 2021 Team Special Weekend
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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
