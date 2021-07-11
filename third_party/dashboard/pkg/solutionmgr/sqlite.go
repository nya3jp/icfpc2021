package solutionmgr

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

type SQLiteManager struct {
	basePath string
	db       *sql.DB
}

func NewSQLiteManager(basePath string) (*SQLiteManager, error) {
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

	return &SQLiteManager{
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

func (m *SQLiteManager) Close() error {
	return m.db.Close()
}

func (m *SQLiteManager) GetProblem(problemID int64) (*Problem, error) {
	var createdAt, minimalDislike int64
	row := m.db.QueryRow("SELECT created_at, minimal_dislike FROM problems WHERE problem_id = ?", problemID)
	if err := row.Scan(&createdAt, &minimalDislike); err != nil {
		return nil, err
	}

	fp := m.problemFilePath(problemID)
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

func (m *SQLiteManager) GetProblems() ([]*Problem, error) {
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

		fp := m.problemFilePath(problemID)
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

func (m *SQLiteManager) UpdateMinimalDislike(problemID int64, dislike int64) error {
	_, err := m.db.Exec("UPDATE problems SET minimal_dislike = ? WHERE problem_id = ?", dislike, problemID)
	if err != nil {
		return err
	}
	return nil
}

func (m *SQLiteManager) GetSolution(solutionID int64) (*Solution, error) {
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

	fp := m.solutionFilePath(fileHash)
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

func (m *SQLiteManager) GetSolutionsForProblem(problemID int64) ([]*Solution, error) {
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

		fp := m.solutionFilePath(fileHash)
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

func (m *SQLiteManager) UpdateSolutionEvalResult(solutionID int64, rejectReason string, dislike int64) error {
	_, err := m.db.Exec("UPDATE solutions SET reject_reason = ?, dislike = ? WHERE solution_id = ?", rejectReason, dislike, solutionID)
	if err != nil {
		return err
	}
	return nil
}

func (m *SQLiteManager) GetSubmittedSolutions() ([]*SubmittedSolution, error) {
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

func (m *SQLiteManager) AddSubmittedSolution(solution *SubmittedSolution) error {
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

func (m *SQLiteManager) problemFilePath(problemID int64) string {
	return filepath.Join(m.basePath, "problems", fmt.Sprintf("%d.json", problemID))
}

func (m *SQLiteManager) solutionFilePath(fileHash string) string {
	return filepath.Join(m.basePath, "solutions", fmt.Sprintf("%s.json", fileHash))
}

func (m *SQLiteManager) AddProblem(problem *Problem) error {
	createdAt := time.Now().Unix()

	if err := problem.Data.Validate(); err != nil {
		return err
	}

	fp := m.problemFilePath(problem.ProblemID)
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

func (m *SQLiteManager) deleteSolution(solutionID int64) error {
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

func (m *SQLiteManager) AddSolution(solution *Solution) (int64, error) {
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

func (m *SQLiteManager) saveSolutionToDisk(data *SolutionData) (string, bool, error) {
	bs, h, err := data.MarshalAndHash()
	if err != nil {
		return "", false, err
	}
	fp := m.solutionFilePath(h)
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
