package solutionmgr

import (
	"crypto/sha256"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	_ "github.com/mattn/go-sqlite3"
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

type ProblemData struct {
	Hole    Hole   `json:"hole"`
	Figure  Figure `json:"figure"`
	Epsilon int64  `json:"epsilon"`
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

type SolutionData struct {
	Vertices []Point `json:"vertices"`
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

type Manager struct {
	basePath string
	db       *sql.DB
}

func NewManager(basePath string) (*Manager, error) {
	dbPath := filepath.Join(basePath, "solutions.db")
	if err := os.MkdirAll(filepath.Dir(dbPath), 0755); err != nil {
		return nil, fmt.Errorf("cannot make directories: %v", err)
	}
	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, fmt.Errorf("cannot open the database: %v", err)
	}
	if err := runMigration(db); err != nil {
		return nil, err
	}

	return &Manager{
		basePath: basePath,
		db:       db,
	}, nil
}

func runMigration(db *sql.DB) error {
	var version int64
	err := db.QueryRow("PRAGMA user_version").Scan(&version)
	if err != nil {
		return fmt.Errorf("cannot get the user_version: %v", err)
	}
	switch version {
	case 0, 1:
		_, err = db.Exec(`
			DROP TABLE IF EXISTS problems;
			DROP TABLE IF EXISTS solutions;
			DROP TABLE IF EXISTS tags;
		`)
		if err != nil {
			return fmt.Errorf("cannot drop tables: %v", err)
		}
		version = 2
		fallthrough
	case 2:
		// This is the initial schema.
		_, err = db.Exec(`
			CREATE TABLE IF NOT EXISTS problems (
				problem_id INTEGER PRIMARY KEY,
				created_at INTEGER NOT NULL,
				minimal_dislike INTEGER NOT NULL
			);
			CREATE TABLE IF NOT EXISTS solutions (
				solution_id INTEGER PRIMARY KEY AUTOINCREMENT,
				problem_id INTEGER NOT NULL,
				created_at INTEGER NOT NULL,
				file_hash STRING NOT NULL,
				dislike INTEGER NOT NULL,
				reject_reason STRING NOT NULL DEFAULT ""
			);
			CREATE TABLE IF NOT EXISTS tags (
				solution_id INTEGER NOT NULL,
				tag TEXT NOT NULL,
				PRIMARY KEY(solution_id, tag)
			);
		`)
		if err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}
		version = 3
		fallthrough
	case 3:
		_, err = db.Exec(`
			CREATE TABLE IF NOT EXISTS submittedsolutions (
				submitted_solution_id TEXT PRIMARY KEY,
				problem_id INTEGER NOT NULL,
				created_at INTEGER NOT NULL,
				solution_id INTEGER NOT NULL
			);
		`)
		if err != nil {
			return fmt.Errorf("cannot add submittedsolutions: %v", err)
		}
		version = 3
		fallthrough
	default:
	}
	// Somehow this cannot accept the template.
	_, err = db.Exec(fmt.Sprintf(`PRAGMA user_version = %d`, version))
	if err != nil {
		return fmt.Errorf("cannot set the schema version %d: %v", version, err)
	}
	return nil
}

func (m *Manager) Close() error {
	return m.db.Close()
}

func (m *Manager) GetProblem(problemID int64) (*Problem, error) {
	var createdAt, minimalDislike int64
	row := m.db.QueryRow("SELECT created_at, minimal_dislike FROM problems WHERE problem_id = ?", problemID)
	if err := row.Scan(&createdAt, &minimalDislike); err != nil {
		return nil, err
	}

	fp := m.ProblemFilePath(problemID)
	b, err := os.ReadFile(fp)
	if err != nil {
		return nil, err
	}
	var data ProblemData
	if err := json.Unmarshal(b, &data); err != nil {
		return nil, err
	}

	return &Problem{
		ProblemID:      problemID,
		CreatedAt:      createdAt,
		MinimalDislike: minimalDislike,
		Data:           data,
	}, nil
}

func (m *Manager) GetProblems() ([]*Problem, error) {
	rows, err := m.db.Query("SELECT problem_id, created_at, minimal_dislike FROM problems")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	problems := make([]*Problem, 0) // must be non-nil
	for rows.Next() {
		var problemID, createdAt, minimalDislike int64
		if err := rows.Scan(&problemID, &createdAt, &minimalDislike); err != nil {
			return nil, err
		}

		fp := m.ProblemFilePath(problemID)
		b, err := os.ReadFile(fp)
		if err != nil {
			return nil, err
		}
		var data ProblemData
		if err := json.Unmarshal(b, &data); err != nil {
			return nil, err
		}

		problems = append(problems, &Problem{
			ProblemID:      problemID,
			CreatedAt:      createdAt,
			MinimalDislike: minimalDislike,
			Data:           data,
		})
	}

	return problems, nil
}

func (m *Manager) UpdateMinimalDislike(problemID int64, dislike int64) error {
	_, err := m.db.Exec("UPDATE problems SET minimal_dislike = ? WHERE problem_id = ?", dislike, problemID)
	if err != nil {
		return err
	}
	return nil
}

func (m *Manager) GetSolution(solutionID int64) (*Solution, error) {
	var fileHash, rejectReason string
	var problemID, createdAt, dislike int64
	row := m.db.QueryRow("SELECT problem_id, created_at, file_hash, dislike, reject_reason FROM solutions WHERE solution_id = ?", solutionID)
	if err := row.Scan(&problemID, &createdAt, &fileHash, &dislike, &rejectReason); err != nil {
		return nil, err
	}

	rows, err := m.db.Query("SELECT tag FROM tags WHERE solution_id = ?", solutionID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	tags := make([]string, 0) // must be non-nil
	for rows.Next() {
		var tag string
		if err := rows.Scan(&tag); err != nil {
			return nil, err
		}
		tags = append(tags, tag)
	}

	fp := m.SolutionFilePath(fileHash)
	b, err := os.ReadFile(fp)
	if err != nil {
		return nil, err
	}
	var data SolutionData
	if err := json.Unmarshal(b, &data); err != nil {
		return nil, err
	}

	return &Solution{
		SolutionID:   solutionID,
		ProblemID:    problemID,
		CreatedAt:    createdAt,
		Dislike:      dislike,
		RejectReason: rejectReason,
		Tags:         tags,
		Data:         data,
		fileHash:     fileHash,
	}, nil
}

func (m *Manager) GetSolutionsForProblem(problemID int64) ([]*Solution, error) {
	rows, err := m.db.Query("SELECT solution_id, created_at, file_hash, dislike, reject_reason FROM solutions WHERE problem_id = ?", problemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	solutionMap := make(map[int64]*Solution)
	for rows.Next() {
		var fileHash, rejectReason string
		var solutionID, createdAt, dislike int64
		if err := rows.Scan(&solutionID, &createdAt, &fileHash, &dislike, &rejectReason); err != nil {
			return nil, err
		}

		fp := m.SolutionFilePath(fileHash)
		b, err := os.ReadFile(fp)
		if err != nil {
			return nil, err
		}
		var data SolutionData
		if err := json.Unmarshal(b, &data); err != nil {
			return nil, err
		}

		solutionMap[solutionID] = &Solution{
			SolutionID:   solutionID,
			ProblemID:    problemID,
			CreatedAt:    createdAt,
			Dislike:      dislike,
			RejectReason: rejectReason,
			Tags:         make([]string, 0), // must be non-nil
			Data:         data,
			fileHash:     fileHash,
		}
	}

	rows, err = m.db.Query("SELECT solution_id, tag FROM tags INNER JOIN solutions USING (solution_id) WHERE solutions.problem_id = ?", problemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var solutionID int64
		var tag string
		if err := rows.Scan(&solutionID, &tag); err != nil {
			return nil, err
		}

		solution := solutionMap[solutionID]
		if solution == nil {
			continue
		}
		solution.Tags = append(solution.Tags, tag)
	}

	solutions := make([]*Solution, 0) // must be non-nil
	for _, s := range solutionMap {
		solutions = append(solutions, s)
	}
	return solutions, nil
}

type SolutionPendingEval struct {
	ProblemID  int64
	SolutionID int64
	FileHash   string
}

func (m *Manager) GetSolutionsPendingEval() ([]*SolutionPendingEval, error) {
	// TODO: Restore this logic.
	/*
		rows, err := m.db.Query("SELECT problem_id, solution_id, file_hash FROM solutions WHERE dislike = ? AND reject_reason IS NULL", DefaultDislike)
		if err != nil {
			return nil, err
		}
		defer rows.Close()

		solutions := make([]*SolutionPendingEval, 0) // must be non-nil
		for rows.Next() {
			var fileHash string
			var problemID, solutionID int64
			if err := rows.Scan(&problemID, &solutionID, &fileHash); err != nil {
				return nil, err
			}

			solutions = append(solutions, &SolutionPendingEval{
				SolutionID: solutionID,
				ProblemID:  problemID,
				FileHash:   fileHash,
			})
		}
		return solutions, nil
	*/
	return nil, nil
}

func (m *Manager) UpdateSolutionEvalResult(solutionID int64, rejectReason string, dislike int64) error {
	_, err := m.db.Exec("UPDATE solutions SET reject_reason = ?, dislike = ? WHERE solution_id = ?", rejectReason, dislike, solutionID)
	if err != nil {
		return err
	}
	return nil
}

func (m *Manager) GetSubmittedSolutions() ([]*SubmittedSolution, error) {
	rows, err := m.db.Query("SELECT submitted_solution_id, problem_id, created_at, solution_id FROM submittedsolutions")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	solutions := make([]*SubmittedSolution, 0) // must be non-nil
	for rows.Next() {
		var submittedSolutionID string
		var problemID, createdAt, solutionID int64
		if err := rows.Scan(&submittedSolutionID, &problemID, &createdAt, &solutionID); err != nil {
			return nil, err
		}

		solutions = append(solutions, &SubmittedSolution{
			ProblemID:           problemID,
			SubmittedSolutionID: submittedSolutionID,
			CreatedAt:           createdAt,
			SolutionID:          solutionID,
		})
	}
	return solutions, nil
}

func (m *Manager) AddSubmittedSolution(solution *SubmittedSolution) error {
	if err := solution.Validate(); err != nil {
		return err
	}

	tx, err := m.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	_, err = tx.Exec(
		"INSERT INTO submittedsolutions(submitted_solution_id, problem_id, created_at, solution_id) VALUES (?, ?, ?, ?)",
		solution.SubmittedSolutionID,
		solution.ProblemID,
		solution.CreatedAt,
		solution.SolutionID,
	)
	if err != nil {
		return err
	}

	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func (m *Manager) ProblemFilePath(problemID int64) string {
	return filepath.Join(m.basePath, "problems", fmt.Sprintf("%d.json", problemID))
}

func (m *Manager) SolutionFilePath(fileHash string) string {
	return filepath.Join(m.basePath, "solutions", fmt.Sprintf("%s.json", fileHash))
}

func (m *Manager) AddProblem(problem *Problem) error {
	createdAt := time.Now().Unix()

	if err := problem.Data.Validate(); err != nil {
		return err
	}

	fp := m.ProblemFilePath(problem.ProblemID)
	if err := os.MkdirAll(filepath.Dir(fp), 0755); err != nil {
		return fmt.Errorf("cannot make directories: %v", err)
	}
	b, err := json.Marshal(problem.Data)
	if err != nil {
		return err
	}
	if err := os.WriteFile(fp, b, 0644); err != nil {
		return fmt.Errorf("cannot write the data file: %v", err)
	}

	tx, err := m.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	_, err = tx.Exec(
		"INSERT INTO problems(problem_id, created_at, minimal_dislike) VALUES (?, ?, ?)",
		problem.ProblemID, createdAt, problem.MinimalDislike,
	)
	if err != nil {
		return err
	}

	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func (m *Manager) deleteSolution(solutionID int64) error {
	_, err := m.db.Exec(`DELETE FROM tags WHERE solution_id = ?`, solutionID)
	if err != nil {
		return err
	}
	_, err = m.db.Exec(`DELETE FROM solutions WHERE solution_id = ?`, solutionID)
	if err != nil {
		return err
	}
	return nil
}

func (m *Manager) AddSolution(solution *Solution) (int64, error) {
	createdAt := time.Now().Unix()
	if solution.CreatedAt != 0 {
		createdAt = solution.CreatedAt
	}

	problem, err := m.GetProblem(solution.ProblemID)
	if err != nil {
		return 0, err
	}

	if err := solution.Data.Validate(&problem.Data); err != nil {
		return 0, err
	}

	h, alreadyExist, err := m.saveSolutionToDisk(&solution.Data)
	if err != nil {
		return 0, err
	}
	if alreadyExist {
		var solutionID int64
		err := m.db.QueryRow("SELECT solution_id FROM solutions WHERE file_hash = ?", h).Scan(&solutionID)
		if err == nil {
			return solutionID, nil
		}
		if err != sql.ErrNoRows {
			return 0, err
		}
		// Continue to create a solution.
	}

	tx, err := m.db.Begin()
	if err != nil {
		return 0, err
	}
	defer tx.Rollback()

	result, err := tx.Exec(
		"INSERT INTO solutions(problem_id, created_at, file_hash, dislike, reject_reason) VALUES (?, ?, ?, ?, ?)",
		solution.ProblemID, createdAt, h, solution.Dislike, solution.RejectReason,
	)
	if err != nil {
		return 0, err
	}

	solutionID, err := result.LastInsertId()
	if err != nil {
		return 0, err
	}
	for _, tag := range solution.Tags {
		if _, err := tx.Exec("INSERT INTO tags(solution_id, tag) VALUES (?, ?) ON CONFLICT(solution_id, tag) DO NOTHING", solutionID, tag); err != nil {
			return 0, err
		}
	}

	if err := tx.Commit(); err != nil {
		return 0, err
	}
	return solutionID, nil
}

func (m *Manager) saveSolutionToDisk(data *SolutionData) (string, bool, error) {
	bs, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return "", false, fmt.Errorf("cannot unmarshal the JSON data: %v", err)
	}
	h := fmt.Sprintf("%x", sha256.Sum256(bs))
	fp := m.SolutionFilePath(h)
	if _, err := os.Lstat(fp); err == nil {
		// Already exists.
		return h, true, nil
	}
	if err := os.MkdirAll(filepath.Dir(fp), 0755); err != nil {
		return "", false, fmt.Errorf("cannot make directories: %v", err)
	}
	if err := os.WriteFile(fp, bs, 0644); err != nil {
		return "", false, fmt.Errorf("cannot write the data file: %v", err)
	}
	return h, false, nil
}
