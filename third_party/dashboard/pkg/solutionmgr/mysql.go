package solutionmgr

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"time"

	_ "github.com/go-sql-driver/mysql"
)

const (
	ICFPC_DATABASE_USER_KEY     = "ICFPC_DATABASE_USER"
	ICFPC_DATABASE_PASSWORD_KEY = "ICFPC_DATABASE_PASSWORD"
	ICFPC_DATABASE_NAME_KEY     = "ICFPC_DATABASE_NAME"
)

func newMySQLConn() (*sql.DB, error) {
	userName, ok := os.LookupEnv(ICFPC_DATABASE_USER_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the DB user")
	}
	password, ok := os.LookupEnv(ICFPC_DATABASE_PASSWORD_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the DB password")
	}
	dbName, ok := os.LookupEnv(ICFPC_DATABASE_NAME_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the DB name")
	}

	db, err := sql.Open("mysql", fmt.Sprintf("%s:%s@tcp(localhost:3306)/%s", userName, password, dbName))
	if err != nil {
		return nil, fmt.Errorf("canot connect to the DB: %v", err)
	}
	db.SetConnMaxLifetime(time.Minute * 3)
	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(10)

	if err := runSchemaMigration(db); err != nil {
		return nil, fmt.Errorf("cannot migrate the DB schema: %v", err)
	}
	return db, nil
}

func runSchemaMigration(db *sql.DB) error {
	// Initialize the schema_version table idempotently.
	if _, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS schema_version (
			id      INTEGER NOT NULL,
			version INTEGER NOT NULL,
			PRIMARY KEY (id)
		) ENGINE=INNODB;
	`); err != nil {
		return fmt.Errorf("cannot create the schema_version table: %v", err)
	}
	if _, err := db.Exec(`INSERT INTO schema_version (id, version) VALUES (1, 0) ON DUPLICATE KEY UPDATE id = 1;`); err != nil {
		return fmt.Errorf("cannot create the schema_version table: %v", err)
	}

	var version int64
	if err := db.QueryRow("SELECT version FROM schema_version WHERE id = 1").Scan(&version); err != nil {
		return fmt.Errorf("cannot get the schema_version: %v", err)
	}

	switch version {
	case 0:
		// This is the initial schema.
		if _, err := db.Exec(`
			CREATE TABLE IF NOT EXISTS problems (
				problem_id      BIGINT NOT NULL,
				created_at      BIGINT NOT NULL,
				minimal_dislike BIGINT NOT NULL,
				data            LONGBLOB NOT NULL,
				PRIMARY KEY (problem_id)
			) ENGINE=INNODB;
		`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}

		if _, err := db.Exec(`
			CREATE TABLE IF NOT EXISTS solutions (
				solution_id   BIGINT NOT NULL AUTO_INCREMENT,
				problem_id    BIGINT NOT NULL,
				created_at    BIGINT NOT NULL,
				file_hash     VARCHAR(256) NOT NULL,
				dislike       BIGINT NOT NULL,
				reject_reason VARCHAR(512) NOT NULL DEFAULT "",
				PRIMARY KEY (solution_id)
			) ENGINE=INNODB;
		`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}
		if _, err := db.Exec(`CREATE UNIQUE INDEX solutions_file_hash ON solutions(file_hash);`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}

		if _, err := db.Exec(`
			CREATE TABLE IF NOT EXISTS solution_data (
				file_hash VARCHAR(256) NOT NULL,
				data      LONGBLOB NOT NULL,
				PRIMARY KEY (file_hash)
			) ENGINE=INNODB;
		`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}

		if _, err := db.Exec(`
			CREATE TABLE IF NOT EXISTS tags (
				solution_id BIGINT NOT NULL,
				tag         VARCHAR(256) NOT NULL,
				PRIMARY KEY(solution_id, tag)
			) ENGINE=INNODB;
		`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}
		if _, err := db.Exec(`CREATE INDEX tags_solution_id ON tags(solution_id);`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}
		if _, err := db.Exec(`CREATE INDEX tags_tag ON tags(tag);`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}

		if _, err := db.Exec(`
			CREATE TABLE IF NOT EXISTS submitted_solutions (
				submitted_solution_id VARCHAR(256),
				problem_id            BIGINT NOT NULL,
				created_at            BIGINT NOT NULL,
				solution_id           BIGINT NOT NULL,
				PRIMARY KEY (submitted_solution_id)
			) ENGINE=INNODB;
		`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}
		version = 1
		fallthrough
	case 1:
		if _, err := db.Exec(`
			CREATE TABLE IF NOT EXISTS running_tasks (
				task_id    BIGINT NOT NULL,
				problem_id BIGINT NOT NULL,
				created_at BIGINT NOT NULL,
				PRIMARY KEY (task_id)
			) ENGINE=INNODB;
		`); err != nil {
			return fmt.Errorf("cannot create tables: %v", err)
		}
		version = 2
		fallthrough
	default:
	}
	if _, err := db.Exec(`UPDATE schema_version SET version = ? WHERE id = 1`, version); err != nil {
		return fmt.Errorf("cannot set the schema version to %d: %v", version, err)
	}
	return nil
}

type MySQLManager struct {
	db *sql.DB
}

func NewMySQLManager() (*MySQLManager, error) {
	db, err := newMySQLConn()
	if err != nil {
		return nil, err
	}
	return &MySQLManager{db}, nil
}

func (m *MySQLManager) Close() error {
	return m.db.Close()
}

func (m *MySQLManager) AddProblem(problem *Problem) error {
	if problem.CreatedAt == 0 {
		problem.CreatedAt = time.Now().Unix()
	}

	if err := problem.Data.Validate(); err != nil {
		return err
	}

	bs, err := json.Marshal(problem.Data)
	if err != nil {
		return err
	}

	if _, err := m.db.Exec(
		"INSERT INTO problems(problem_id, created_at, minimal_dislike, data) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE data = ?",
		problem.ProblemID, problem.CreatedAt, problem.MinimalDislike, bs, bs,
	); err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) AddRunningTask(taskID, problemID int64) error {
	createdAt := time.Now().Unix()

	if _, err := m.db.Exec(
		"INSERT INTO running_tasks(task_id, problem_id, created_at) VALUES (?, ?, ?)",
		taskID, problemID, createdAt,
	); err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) AddSolution(solution *Solution) (int64, error) {
	if solution.CreatedAt != 0 {
		solution.CreatedAt = time.Now().Unix()
	}

	problem, err := m.GetProblem(solution.ProblemID)
	if err != nil {
		return 0, err
	}

	if err := solution.Data.Validate(&problem.Data); err != nil {
		return 0, err
	}

	bs, fileHash, err := solution.Data.MarshalAndHash()
	if err != nil {
		return 0, err
	}

	tx, err := m.db.Begin()
	if err != nil {
		return 0, err
	}
	defer tx.Rollback()

	var solutionID int64
	if err := tx.QueryRow("SELECT solution_id FROM solutions WHERE file_hash = ?", fileHash).Scan(&solutionID); err == nil {
		// Already exist.
		return solutionID, nil
	} else if err != sql.ErrNoRows {
		return 0, err
	}

	if _, err := tx.Exec("INSERT INTO solution_data(file_hash, data) VALUES (?, ?) ON DUPLICATE KEY UPDATE file_hash = ?", fileHash, bs, fileHash); err != nil {
		return 0, err
	}

	if solution.SolutionID != 0 {
		_, err := tx.Exec(
			"INSERT INTO solutions(solution_id, problem_id, created_at, file_hash, dislike, reject_reason) VALUES (?, ?, ?, ?, ?, ?)",
			solution.SolutionID, solution.ProblemID, solution.CreatedAt, fileHash, solution.Dislike, solution.RejectReason,
		)
		if err != nil {
			return 0, err
		}
		solutionID = solution.SolutionID
	} else {
		result, err := tx.Exec(
			"INSERT INTO solutions(problem_id, created_at, file_hash, dislike, reject_reason) VALUES (?, ?, ?, ?, ?)",
			solution.ProblemID, solution.CreatedAt, fileHash, solution.Dislike, solution.RejectReason,
		)
		if err != nil {
			return 0, err
		}
		solutionID, err = result.LastInsertId()
		if err != nil {
			return 0, err
		}
	}
	for _, tag := range solution.Tags {
		if _, err := tx.Exec("INSERT INTO tags(solution_id, tag) VALUES (?, ?) ON DUPLICATE KEY UPDATE solution_id = ?", solutionID, tag, solutionID); err != nil {
			return 0, err
		}
	}

	if err := tx.Commit(); err != nil {
		return 0, err
	}
	return solutionID, nil
}

func (m *MySQLManager) AddSolutionTag(solutionID int64, tag string) error {
	if _, err := m.db.Exec("INSERT INTO tags(solution_id, tag) VALUES (?, ?) ON DUPLICATE KEY UPDATE solution_id = ?", solutionID, tag, solutionID); err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) SetSolutionAutoIncrement() error {
	var solutionID int64
	if err := m.db.QueryRow("SELECT MAX(solution_id) FROM solutions").Scan(&solutionID); err != nil {
		return err
	}
	if _, err := m.db.Exec(fmt.Sprintf("ALTER TABLE solutions AUTO_INCREMENT = %d", solutionID+1)); err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) AddSubmittedSolution(solution *SubmittedSolution) error {
	if err := solution.Validate(); err != nil {
		return err
	}

	if _, err := m.db.Exec(
		"INSERT INTO submitted_solutions(submitted_solution_id, problem_id, created_at, solution_id) VALUES (?, ?, ?, ?)",
		solution.SubmittedSolutionID,
		solution.ProblemID,
		solution.CreatedAt,
		solution.SolutionID,
	); err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) GetProblem(problemID int64) (*Problem, error) {
	var createdAt, minimalDislike int64
	var bs []byte
	if err := m.db.QueryRow(
		"SELECT created_at, minimal_dislike, data FROM problems WHERE problem_id = ?",
		problemID,
	).Scan(&createdAt, &minimalDislike, &bs); err != nil {
		return nil, err
	}

	var data ProblemData
	if err := json.Unmarshal(bs, &data); err != nil {
		return nil, err
	}

	return &Problem{
		ProblemID:      problemID,
		CreatedAt:      createdAt,
		MinimalDislike: minimalDislike,
		Data:           data,
	}, nil
}

func (m *MySQLManager) GetProblems() ([]*Problem, error) {
	rows, err := m.db.Query("SELECT problem_id, created_at, minimal_dislike, data FROM problems")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	problems := make([]*Problem, 0) // must be non-nil
	for rows.Next() {
		var problemID, createdAt, minimalDislike int64
		var bs []byte
		if err := rows.Scan(&problemID, &createdAt, &minimalDislike, &bs); err != nil {
			return nil, err
		}

		var data ProblemData
		if err := json.Unmarshal(bs, &data); err != nil {
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

func (m *MySQLManager) GetRunningTasks() ([]*RunningTask, error) {
	rows, err := m.db.Query("SELECT task_id, problem_id, created_at FROM running_tasks")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	tasks := make([]*RunningTask, 0) // must be non-nil
	for rows.Next() {
		var taskID, problemID, createdAt int64
		if err := rows.Scan(&taskID, &problemID, &createdAt); err != nil {
			return nil, err
		}

		tasks = append(tasks, &RunningTask{
			TaskID:    taskID,
			ProblemID: problemID,
			CreatedAt: createdAt,
		})
	}
	return tasks, nil
}

func (m *MySQLManager) GetSolution(solutionID int64) (*Solution, error) {
	var fileHash, rejectReason string
	var problemID, createdAt, dislike int64
	var bs []byte
	if err := m.db.QueryRow(
		"SELECT problem_id, created_at, file_hash, dislike, reject_reason, data FROM solutions INNER JOIN solution_data USING (file_hash) WHERE solution_id = ?",
		solutionID,
	).Scan(&problemID, &createdAt, &fileHash, &dislike, &rejectReason, &bs); err != nil {
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

	var data SolutionData
	if err := json.Unmarshal(bs, &data); err != nil {
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

func (m *MySQLManager) GetSolutionsForProblem(problemID int64) ([]*Solution, error) {
	rows, err := m.db.Query("SELECT solution_id, created_at, file_hash, dislike, reject_reason, data FROM solutions INNER JOIN solution_data USING (file_hash) WHERE problem_id = ?", problemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	solutionMap := make(map[int64]*Solution)
	for rows.Next() {
		var fileHash, rejectReason string
		var solutionID, createdAt, dislike int64
		var bs []byte
		if err := rows.Scan(&solutionID, &createdAt, &fileHash, &dislike, &rejectReason, &bs); err != nil {
			return nil, err
		}

		var data SolutionData
		if err := json.Unmarshal(bs, &data); err != nil {
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

func (m *MySQLManager) GetSolutionsForTag(tag string) ([]*Solution, error) {
	rows, err := m.db.Query("SELECT solution_id, problem_id, created_at, file_hash, dislike, reject_reason, data FROM solutions INNER JOIN solution_data USING (file_hash) INNER JOIN tags USING (solution_id) WHERE tag = ?", tag)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	solutionMap := make(map[int64]*Solution)
	for rows.Next() {
		var fileHash, rejectReason string
		var solutionID, problemID, createdAt, dislike int64
		var bs []byte
		if err := rows.Scan(&solutionID, &problemID, &createdAt, &fileHash, &dislike, &rejectReason, &bs); err != nil {
			return nil, err
		}

		var data SolutionData
		if err := json.Unmarshal(bs, &data); err != nil {
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

	rows, err = m.db.Query("SELECT solution_id, tag FROM tags WHERE solution_id IN (SELECT solution_id FROM tags WHERE tag = ?)", tag)
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

func (m *MySQLManager) GetSubmittedSolutions() ([]*SubmittedSolution, error) {
	rows, err := m.db.Query("SELECT submitted_solution_id, problem_id, created_at, solution_id FROM submitted_solutions")
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

func (m *MySQLManager) RemoveSolutionTag(solutionID int64, tag string) error {
	_, err := m.db.Exec("DELETE FROM tags WHERE solution_id = ? AND tag = ?", solutionID, tag)
	if err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) UpdateMinimalDislike(problemID int64, dislike int64) error {
	_, err := m.db.Exec("UPDATE problems SET minimal_dislike = ? WHERE problem_id = ?", dislike, problemID)
	if err != nil {
		return err
	}
	return nil
}

func (m *MySQLManager) UpdateSolutionEvalResult(solutionID int64, rejectReason string, dislike int64) error {
	_, err := m.db.Exec("UPDATE solutions SET reject_reason = ?, dislike = ? WHERE solution_id = ?", rejectReason, dislike, solutionID)
	if err != nil {
		return err
	}
	return nil
}
