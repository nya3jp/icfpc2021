package eval

import (
	"context"
	"encoding/json"
	"log"
	"os/exec"
	"time"

	"icfpc2021/dashboard/pkg/solutionmgr"
)

func UpdateDislikeTask(ctx context.Context, scorerPath string, mgr *solutionmgr.Manager, ch <-chan bool) {
	tick := time.NewTicker(time.Minute)
	defer tick.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-tick.C:
			if err := UpdateDislikes(scorerPath, mgr); err != nil {
				log.Printf("Failed to update dislikes: %v", err)
			}
		case <-ch:
			if err := UpdateDislikes(scorerPath, mgr); err != nil {
				log.Printf("Failed to update dislikes: %v", err)
			}
		}
	}
}

func UpdateDislikes(scorerPath string, mgr *solutionmgr.Manager) error {
	solutions, err := mgr.GetSolutionsPendingEval()
	if err != nil {
		return err
	}
	for _, solution := range solutions {
		valid, dislike, err := eval(scorerPath, mgr.ProblemFilePath(solution.ProblemID), mgr.SolutionFilePath(solution.FileHash))
		if err != nil {
			return err
		}
		if err := mgr.UpdateSolutionEvalResult(solution.SolutionID, valid, dislike); err != nil {
			return err
		}
	}
	return nil
}

type scorerOutput struct {
	IsValid bool  `json:"is_valid"`
	Dislike int64 `json:"dislike"`
}

func eval(scorerPath string, problemPath, solutionPath string) (bool, int64, error) {
	cmd := exec.Command(
		scorerPath,
		problemPath,
		solutionPath,
		"json",
	)
	bs, err := cmd.Output()
	if err != nil {
		return false, 0, err
	}
	var output scorerOutput
	if err := json.Unmarshal(bs, &output); err != nil {
		return false, 0, err
	}

	return output.IsValid, output.Dislike, nil
}
