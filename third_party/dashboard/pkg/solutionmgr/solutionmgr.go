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

const DefaultDislike = 999999999

type Problem struct {
	ProblemID int64           `json:"problem_id"`
	CreatedAt int64           `json:"created_at"`
	Data      json.RawMessage `json:"data"`
}

type Solution struct {
	SolutionID int64           `json:"solution_id"`
	ProblemID  int64           `json:"problem_id"`
	CreatedAt  int64           `json:"created_at"`
	Dislike    int64           `json:"dislike,omitempty"`
	Tags       []string        `json:"tags"`
	Data       json.RawMessage `json:"data"`
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
	case 0:
		// This is the initial schema.
		_, err = db.Exec(`
			CREATE TABLE IF NOT EXISTS problems (
				problem_id INTEGER PRIMARY KEY,
				created_at INTEGER NOT NULL
			);
			CREATE TABLE IF NOT EXISTS solutions (
				solution_id INTEGER PRIMARY KEY AUTOINCREMENT,
				problem_id INTEGER NOT NULL,
				created_at INTEGER NOT NULL,
				file_hash STRING NOT NULL,
				dislike INTEGER NOT NULL
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
		_, err = db.Exec(`
			ALTER TABLE solutions ADD is_invalid INTEGER;
		`)
		if err != nil {
			return fmt.Errorf("cannot update to the schema version %d: %v", version+1, err)
		}
		version = 1
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
	var createdAt int64
	row := m.db.QueryRow("SELECT created_at FROM problems WHERE problem_id = ?", problemID)
	if err := row.Scan(&createdAt); err != nil {
		return nil, err
	}

	fp := m.ProblemFilePath(problemID)
	data, err := os.ReadFile(fp)
	if err != nil {
		return nil, err
	}

	return &Problem{
		ProblemID: problemID,
		CreatedAt: createdAt,
		Data:      data,
	}, nil
}

func (m *Manager) GetProblems() ([]*Problem, error) {
	rows, err := m.db.Query("SELECT problem_id, created_at FROM problems")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	problems := make([]*Problem, 0) // must be non-nil
	for rows.Next() {
		var problemID, createdAt int64
		if err := rows.Scan(&problemID, &createdAt); err != nil {
			return nil, err
		}

		fp := m.ProblemFilePath(problemID)
		data, err := os.ReadFile(fp)
		if err != nil {
			return nil, err
		}

		problems = append(problems, &Problem{
			ProblemID: problemID,
			CreatedAt: createdAt,
			Data:      data,
		})
	}

	return problems, nil
}

func (m *Manager) GetSolution(solutionID int64) (*Solution, error) {
	var fileHash string
	var problemID, createdAt, dislike int64
	row := m.db.QueryRow("SELECT problem_id, created_at, file_hash, dislike FROM solutions WHERE solution_id = ?", solutionID)
	if err := row.Scan(&problemID, &createdAt, &fileHash, &dislike); err != nil {
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
	data, err := os.ReadFile(fp)
	if err != nil {
		return nil, err
	}

	return &Solution{
		SolutionID: solutionID,
		ProblemID:  problemID,
		CreatedAt:  createdAt,
		Dislike:    dislike,
		Tags:       tags,
		Data:       data,
	}, nil
}

func (m *Manager) GetSolutionsForProblem(problemID int64) ([]*Solution, error) {
	rows, err := m.db.Query("SELECT solution_id, created_at, file_hash, dislike FROM solutions WHERE problem_id = ?", problemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	solutionMap := make(map[int64]*Solution)
	for rows.Next() {
		var fileHash string
		var solutionID, createdAt, dislike int64
		if err := rows.Scan(&solutionID, &createdAt, &fileHash, &dislike); err != nil {
			return nil, err
		}

		fp := m.SolutionFilePath(fileHash)
		data, err := os.ReadFile(fp)
		if err != nil {
			return nil, err
		}

		solutionMap[solutionID] = &Solution{
			SolutionID: solutionID,
			ProblemID:  problemID,
			CreatedAt:  createdAt,
			Dislike:    dislike,
			Tags:       make([]string, 0), // must be non-nil
			Data:       data,
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
	rows, err := m.db.Query("SELECT problem_id, solution_id, file_hash FROM solutions WHERE dislike = ? AND is_invalid IS NULL", DefaultDislike)
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
}

func (m *Manager) UpdateSolutionEvalResult(solutionID int64, isInvalid bool, dislike int64) error {
	_, err := m.db.Exec("UPDATE solutions SET is_invalid = ?, dislike = ? WHERE solution_id = ?", isInvalid, dislike, solutionID)
	if err != nil {
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

	fp := m.ProblemFilePath(problem.ProblemID)
	if _, err := os.Lstat(fp); err == nil {
		return fmt.Errorf("already exists")
	}
	if err := os.MkdirAll(filepath.Dir(fp), 0755); err != nil {
		return fmt.Errorf("cannot make directories: %v", err)
	}
	if err := os.WriteFile(fp, problem.Data, 0644); err != nil {
		return fmt.Errorf("cannot write the data file: %v", err)
	}

	tx, err := m.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	_, err = tx.Exec(
		"INSERT INTO problems(problem_id, created_at) VALUES (?, ?)",
		problem.ProblemID, createdAt,
	)
	if err != nil {
		return err
	}

	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func (m *Manager) AddSolution(solution *Solution) error {
	createdAt := time.Now().Unix()

	h, err := m.saveSolutionToDisk(solution.Data)
	if err != nil {
		return err
	}

	tx, err := m.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	result, err := tx.Exec(
		"INSERT INTO solutions(problem_id, created_at, file_hash, dislike) VALUES (?, ?, ?, ?)",
		solution.ProblemID, createdAt, h, DefaultDislike,
	)
	if err != nil {
		return err
	}

	solutionID, err := result.LastInsertId()
	if err != nil {
		return err
	}
	for _, tag := range solution.Tags {
		if _, err := tx.Exec("INSERT INTO tags(solution_id, tag) VALUES (?, ?) ON CONFLICT(solution_id, tag) DO NOTHING", solutionID, tag); err != nil {
			return err
		}
	}

	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func (m *Manager) saveSolutionToDisk(solutionJSON []byte) (string, error) {
	var d json.RawMessage
	if err := json.Unmarshal(solutionJSON, &d); err != nil {
		return "", fmt.Errorf("cannot parse the JSON data: %v", err)
	}
	bs, err := json.MarshalIndent(d, "", "  ")
	if err != nil {
		return "", fmt.Errorf("cannot unmarshal the JSON data: %v", err)
	}
	h := fmt.Sprintf("%x", sha256.Sum256(bs))
	fp := m.SolutionFilePath(h)
	if _, err := os.Lstat(fp); err == nil {
		// Already exists.
		return h, nil
	}
	if err := os.MkdirAll(filepath.Dir(fp), 0755); err != nil {
		return "", fmt.Errorf("cannot make directories: %v", err)
	}
	if err := os.WriteFile(fp, bs, 0644); err != nil {
		return "", fmt.Errorf("cannot write the data file: %v", err)
	}
	return h, nil
}
