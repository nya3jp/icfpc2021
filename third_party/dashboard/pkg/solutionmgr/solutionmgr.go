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

type Solution struct {
	ProblemID  string `json:"problem_id"`
	SolutionID string `json:"solution_id"`
	CreatedAt  int64  `json:"created_at"`
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
	_, err = db.Exec(`
			CREATE TABLE IF NOT EXISTS solutions (
				problem_id TEXT NOT NULL,
				solution_id TEXT NOT NULL,
				created_at INTEGER NOT NULL,
				PRIMARY KEY(problem_id, solution_id)
			);
			CREATE TABLE IF NOT EXISTS tags (
				problem_id TEXT NOT NULL,
				solution_id TEXT NOT NULL,
				tag TEXT NOT NULL,
				PRIMARY KEY(problem_id, solution_id, tag)
			);
			CREATE TABLE IF NOT EXISTS solution_sets (
				solution_set TEXT NOT NULL,
				created_at INTEGER NOT NULL,
				PRIMARY KEY(solution_set)
			);
			CREATE TABLE IF NOT EXISTS solution_set_assocs (
				solution_set TEXT NOT NULL,
				problem_id TEXT NOT NULL,
				solution_id TEXT NOT NULL,
				PRIMARY KEY(solution_set, problem_id, solution_id)
			);
		`)
	if err != nil {
		return nil, fmt.Errorf("cannot create tables: %v", err)
	}

	return &Manager{
		basePath: basePath,
		db:       db,
	}, nil
}

func (m *Manager) Close() error {
	return m.db.Close()
}

func (m *Manager) GetRecentSolutions(page, limit int) ([]*Solution, error) {
	rows, err := m.db.Query("SELECT problem_id, solution_id, created_at FROM solutions ORDER BY created_at DESC LIMIT ? OFFSET ?", limit, page*limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var ret []*Solution
	for rows.Next() {
		var problemID string
		var solutionID string
		var createdAt int64
		err = rows.Scan(&problemID, &solutionID, &createdAt)
		if err != nil {
			return nil, err
		}
		ret = append(ret, &Solution{
			ProblemID:  problemID,
			SolutionID: solutionID,
			CreatedAt:  createdAt,
		})
	}
	err = rows.Err()
	if err != nil {
		return nil, err
	}
	return ret, nil
}

func (m *Manager) Add(problemID string, solutionJSON []byte, tags []string, solutionSet string) error {
	createdAt := time.Now().Unix()
	h, _, err := m.saveToDisk(problemID, solutionJSON)
	if err != nil {
		return err
	}
	tx, err := m.db.Begin()
	if err != nil {
		return err
	}
	_, err = tx.Exec("INSERT INTO solutions(problem_id, solution_id, created_at) VALUES (?, ?, ?) ON CONFLICT(problem_id, solution_id) DO NOTHING", problemID, h, createdAt)
	if err != nil {
		return err
	}
	for _, tag := range tags {
		_, err = tx.Exec("INSERT INTO tags(problem_id, solution_id, tag) VALUES (?, ?, ?) ON CONFLICT(problem_id, solution_id, tag) DO NOTHING", problemID, h, tag)
		if err != nil {
			return err
		}
	}
	if solutionSet != "" {
		_, err = tx.Exec("INSERT INTO solution_sets(solution_set, created_at) VALUES (?, ?) ON CONFLICT(solution_set) DO NOTHING", solutionSet, createdAt)
		if err != nil {
			return err
		}
		_, err = tx.Exec("INSERT INTO solution_set_assocs(solution_set, problem_id, solution_id) VALUES (?, ?, ?) ON CONFLICT(solution_set, problem_id, solution_id) DO NOTHING", solutionSet, problemID, h)
		if err != nil {
			return err
		}
	}
	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func (m *Manager) saveToDisk(problemID string, solutionJSON []byte) (string, bool, error) {
	var d json.RawMessage
	if err := json.Unmarshal(solutionJSON, &d); err != nil {
		return "", false, fmt.Errorf("cannot parse the solution: %v", err)
	}
	bs, err := json.MarshalIndent(d, "", "  ")
	if err != nil {
		return "", false, fmt.Errorf("cannot unmarshal the solution: %v", err)
	}
	h := fmt.Sprintf("%x.json", sha256.Sum256(bs))
	fp := filepath.Join(m.basePath, fmt.Sprintf("solution-%s", problemID), h)
	if _, err := os.Lstat(fp); err == nil {
		// Already exists.
		return h, false, nil
	}
	if err := os.MkdirAll(filepath.Dir(fp), 0755); err != nil {
		return "", false, fmt.Errorf("cannot make directories: %v", err)
	}
	if err := os.WriteFile(fp, bs, 0644); err != nil {
		return "", false, fmt.Errorf("cannot write the solution: %v", err)
	}
	return h, true, nil
}
