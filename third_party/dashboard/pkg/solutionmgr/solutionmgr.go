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

const defaultDislike = 999999999

type Solution struct {
	SolutionID int64    `json:"solution_id"`
	ProblemID  int64    `json:"problem_id"`
	CreatedAt  int64    `json:"created_at"`
	Dislike    int64    `json:"dislike"`
	Tags       []string `json:"tags"`
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
	_, err := db.Exec(`
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
	return nil
}

func (m *Manager) Close() error {
	return m.db.Close()
}

func (m *Manager) GetSolution(solutionID int64) ([]byte, error) {
	var fileHash string
	err := m.db.QueryRow("SELECT file_hash FROM solutions WHERE solution_id = ?", solutionID).Scan(&fileHash)
	if err != nil {
		return nil, err
	}
	fp := filepath.Join(m.basePath, "solutions", fmt.Sprintf("%s.json", fileHash))
	return os.ReadFile(fp)
}

func (m *Manager) GetSolutionMetadata(solutionID int64) (*Solution, error) {
	rows, err := m.db.Query("SELECT problem_id, created_at, dislike FROM solutions WHERE solution_id = ?", solutionID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var problemID int64
		var createdAt int64
		var dislike int64
		err = rows.Scan(&problemID, &createdAt, &dislike)
		if err != nil {
			return nil, err
		}
		tags, err := m.getTags(solutionID)
		if err != nil {
			return nil, err
		}
		return &Solution{
			SolutionID: solutionID,
			ProblemID:  problemID,
			CreatedAt:  createdAt,
			Tags:       tags,
		}, nil
	}
	err = rows.Err()
	if err != nil {
		return nil, err
	}
	return nil, fmt.Errorf("not found")
}

func (m *Manager) getTags(solutionID int64) ([]string, error) {
	rows, err := m.db.Query("SELECT tag FROM tags WHERE solution_id = ?", solutionID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var ret []string
	for rows.Next() {
		var tag string
		err = rows.Scan(&tag)
		if err != nil {
			return nil, err
		}
		ret = append(ret, tag)
	}
	err = rows.Err()
	if err != nil {
		return nil, err
	}
	return ret, nil
}

func (m *Manager) AddSolution(problemID int64, solutionJSON []byte, tags []string) error {
	createdAt := time.Now().Unix()
	h, err := m.saveToDisk(problemID, solutionJSON)
	if err != nil {
		return err
	}
	tx, err := m.db.Begin()
	if err != nil {
		return err
	}
	result, err := tx.Exec(
		"INSERT INTO solutions(problem_id, created_at, file_hash, dislike) VALUES (?, ?, ?, ?)",
		problemID, createdAt, h, defaultDislike,
	)
	if err != nil {
		return err
	}
	solutionID, err := result.LastInsertId()
	if err != nil {
		return err
	}
	for _, tag := range tags {
		_, err = tx.Exec("INSERT INTO tags(solution_id, tag) VALUES (?, ?) ON CONFLICT(solution_id, tag) DO NOTHING", solutionID, tag)
		if err != nil {
			return err
		}
	}
	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func (m *Manager) saveToDisk(problemID int64, solutionJSON []byte) (string, error) {
	var d json.RawMessage
	if err := json.Unmarshal(solutionJSON, &d); err != nil {
		return "", fmt.Errorf("cannot parse the solution: %v", err)
	}
	bs, err := json.MarshalIndent(d, "", "  ")
	if err != nil {
		return "", fmt.Errorf("cannot unmarshal the solution: %v", err)
	}
	h := fmt.Sprintf("%x", sha256.Sum256(bs))
	fp := filepath.Join(m.basePath, "solutions", fmt.Sprintf("%s.json", h))
	if _, err := os.Lstat(fp); err == nil {
		// Already exists.
		return h, nil
	}
	if err := os.MkdirAll(filepath.Dir(fp), 0755); err != nil {
		return "", fmt.Errorf("cannot make directories: %v", err)
	}
	if err := os.WriteFile(fp, bs, 0644); err != nil {
		return "", fmt.Errorf("cannot write the solution: %v", err)
	}
	return h, nil
}
