package solutionmgr

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
)

type Point []int64

func (p Point) Validate() error {
	if len(p) != 2 {
		return fmt.Errorf("bad point: got %d elems, want 2 elems", len(p))
	}
	return nil
}

type Edge []int64

type Hole []Point

func (h Hole) Validate() error {
	for _, p := range h {
		if err := p.Validate(); err != nil {
			return err
		}
	}
	return nil
}

type Figure struct {
	Edges    []Edge  `json:"edges"`
	Vertices []Point `json:"vertices"`
}

func (f *Figure) Validate() error {
	for _, edge := range f.Edges {
		if len(edge) != 2 {
			return fmt.Errorf("bad edge: got %d elems, want 2 elems", len(edge))
		}
		for _, e := range edge {
			if e < 0 || e >= int64(len(f.Vertices)) {
				return fmt.Errorf("edge index out of range: got %d, want (0, %d)", e, len(f.Vertices))
			}
		}
	}
	for _, p := range f.Vertices {
		if err := p.Validate(); err != nil {
			return err
		}
	}
	return nil
}

type Bonus struct {
	Position Point  `json:"position"`
	Bonus    string `json:"bonus"`
	Problem  int64  `json:"problem"`
}

type ProblemData struct {
	Hole    Hole    `json:"hole"`
	Figure  Figure  `json:"figure"`
	Epsilon int64   `json:"epsilon"`
	Bonus   []Bonus `json:"bonuses"`
}

func (d *ProblemData) Validate() error {
	if err := d.Hole.Validate(); err != nil {
		return err
	}
	if err := d.Figure.Validate(); err != nil {
		return err
	}
	if d.Epsilon < 0 {
		return fmt.Errorf("negative epsilon: %d", d.Epsilon)
	}
	return nil
}

type Problem struct {
	ProblemID      int64       `json:"problem_id"`
	CreatedAt      int64       `json:"created_at"`
	MinimalDislike int64       `json:"minimal_dislike"`
	Data           ProblemData `json:"data"`
}

type UsedBonus struct {
	Bonus   string `json:"bonus"`
	Problem int64  `json:"problem"`
}

type SolutionData struct {
	Vertices []Point     `json:"vertices"`
	Bonuses  []UsedBonus `json:"bonuses"`
}

func (s *SolutionData) MarshalAndHash() ([]byte, string, error) {
	bs, err := json.MarshalIndent(s, "", "  ")
	if err != nil {
		return nil, "", fmt.Errorf("cannot unmarshal the JSON data: %v", err)
	}
	h := fmt.Sprintf("%x", sha256.Sum256(bs))
	return bs, h, nil
}

func (s *SolutionData) Validate(problem *ProblemData) error {
	if len(s.Vertices) != len(problem.Figure.Vertices) {
		return fmt.Errorf("wrong number of vertices: got %d, want %d", len(s.Vertices), len(problem.Figure.Vertices))
	}
	return nil
}

type Solution struct {
	SolutionID   int64        `json:"solution_id"`
	ProblemID    int64        `json:"problem_id"`
	CreatedAt    int64        `json:"created_at"`
	Dislike      int64        `json:"dislike"`
	RejectReason string       `json:"reject_reason"`
	Tags         []string     `json:"tags"`
	Data         SolutionData `json:"data"`
	fileHash     string
}

type SubmittedSolution struct {
	ProblemID           int64  `json:"problem_id"`
	SubmittedSolutionID string `json:"submitted_solution_id"`
	CreatedAt           int64  `json:"created_at"`
	SolutionID          int64  `json:"solution_id"`
}

type RunningTask struct {
	TaskID    int64 `json:"task_id"`
	ProblemID int64 `json:"problem_id"`
	CreatedAt           int64  `json:"created_at"`
}

func (s *SubmittedSolution) Validate() error {
	if s.ProblemID == 0 {
		return fmt.Errorf("empty problem ID: %d", s.ProblemID)
	}
	if s.SubmittedSolutionID == "" {
		return fmt.Errorf("empty submitted solution ID: %s", s.SubmittedSolutionID)
	}
	if s.CreatedAt == 0 {
		return fmt.Errorf("empty creation time: %d", s.CreatedAt)
	}
	if s.SolutionID == 0 {
		return fmt.Errorf("empty solution ID: %d", s.SolutionID)
	}
	return nil
}
