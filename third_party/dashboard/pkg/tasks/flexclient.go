package tasks

import (
	"database/sql"
	"fmt"
	"os"
	"time"

	"github.com/nya3jp/flex"
	"github.com/nya3jp/flex/flexpb"
	"google.golang.org/protobuf/types/known/durationpb"
)

const (
	ICFPC_DATABASE_USER_KEY      = "ICFPC_DATABASE_USER"
	ICFPC_DATABASE_PASSWORD_KEY  = "ICFPC_DATABASE_PASSWORD"
	ICFPC_FLEX_DATABASE_NAME_KEY = "ICFPC_FLEX_DATABASE_NAME"
)

func NewFlexClient() (*flex.Client, error) {
	userName, ok := os.LookupEnv(ICFPC_DATABASE_USER_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the DB user")
	}
	password, ok := os.LookupEnv(ICFPC_DATABASE_PASSWORD_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the DB password")
	}
	dbName, ok := os.LookupEnv(ICFPC_FLEX_DATABASE_NAME_KEY)
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

	return flex.NewClient(db), nil
}

func NewTask(deadline, timeLimit time.Duration, penaltyRatio, problemID int64, bonus string) *flexpb.TaskSpec {
	shell := fmt.Sprintf("./tanakh-solver solve --no-submit --time-limit=%d --penalty-ratio=%d", deadline/time.Second, penaltyRatio)
	if bonus != "" {
		shell += fmt.Sprintf(" ---use-bonus %s", bonus)
	}
	shell += fmt.Sprintf(" %d", problemID)

	return &flexpb.TaskSpec{
		Command: &flexpb.TaskCommand{
			Shell: shell,
		},
		Constraints: &flexpb.TaskConstraints{
			Priority: 1000,
		},
		Limits: &flexpb.TaskLimits{Time: durationpb.New(deadline)},
		Packages: []*flexpb.TaskPackage{
			{
				Url: "https://storage.googleapis.com/special-weekend-2021-flex/packages/tanakh-solver.16e9e4c.tar.gz",
			},
		},
	}

}
