package eval

import (
	"context"
	"encoding/json"
	"log"
	"os/exec"
	"time"

	"icfpc2021/dashboard/pkg/solutionmgr"
)

const RejectDislike = 999999999

func UpdateDislikeTask(ctx context.Context, scorerPath string, mgr *solutionmgr.Manager) {
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
		}
	}
}

func UpdateDislikes(scorerPath string, mgr *solutionmgr.Manager) error {
	solutions, err := mgr.GetSolutionsPendingEval()
	if err != nil {
		return err
	}
	for _, solution := range solutions {
		dislike, rejectReason := Eval(scorerPath, mgr.ProblemFilePath(solution.ProblemID), mgr.SolutionFilePath(solution.FileHash))
		if err := mgr.UpdateSolutionEvalResult(solution.SolutionID, rejectReason, dislike); err != nil {
			return err
		}
	}
	return nil
}

type scorerOutput struct {
	IsValid bool  `json:"is_valid"`
	Dislike int64 `json:"dislike"`
}

func Eval(scorerPath string, problemPath, solutionPath string) (int64, string) {
	cmd := exec.Command(
		scorerPath,
		problemPath,
		solutionPath,
		"json",
	)
	bs, err := cmd.Output()
	if err != nil {
		return RejectDislike, err.Error()
	}
	var output scorerOutput
	if err := json.Unmarshal(bs, &output); err != nil {
		return RejectDislike, err.Error()
	}
	if !output.IsValid {
		return RejectDislike, "rejected by scorer"
	}
	return output.Dislike, ""
}
