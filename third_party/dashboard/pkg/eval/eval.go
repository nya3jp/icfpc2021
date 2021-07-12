package eval

import (
	"encoding/json"
	"flag"
	"io/ioutil"
	"os"
	"os/exec"

	"icfpc2021/dashboard/pkg/solutionmgr"
)

var (
	scorerPath = flag.String("scorer_path", "/static/scorer", "")
)

const RejectDislike = 999999999

func Recalculate(mgr *solutionmgr.MySQLManager) error {
	problems, err := mgr.GetProblems()
	if err != nil {
		return err
	}
	for _, problem := range problems {
		solutions, err := mgr.GetSolutionsForProblem(problem.ProblemID)
		if err != nil {
			return err
		}
		for _, solution := range solutions {
			dislike, acquiredBonus, rejectReason, err := EvalData(&problem.Data, &solution.Data)
			if err != nil {
				return err
			}
			if err := mgr.UpdateSolutionEvalResult(solution.SolutionID, rejectReason, dislike, acquiredBonus); err != nil {
				return err
			}
		}
	}
	return nil
}

type scorerOutput struct {
	IsValid       bool                    `json:"is_valid"`
	Dislike       int64                   `json:"dislike"`
	AcquiredBonus []solutionmgr.UsedBonus `json:"bonus"`
}

func EvalData(problemData *solutionmgr.ProblemData, solutionData *solutionmgr.SolutionData) (int64, []solutionmgr.UsedBonus, string, error) {
	emptyBonuses := make([]solutionmgr.UsedBonus, 0)
	tmpProblem, err := ioutil.TempFile("", "scorerproblem.")
	if err != nil {
		return 0, emptyBonuses, "", err
	}
	defer tmpProblem.Close()
	defer os.Remove(tmpProblem.Name())
	if err := json.NewEncoder(tmpProblem).Encode(problemData); err != nil {
		return 0, emptyBonuses, "", err
	}

	tmpSolution, err := ioutil.TempFile("", "scorersolution.")
	if err != nil {
		return 0, emptyBonuses, "", err
	}
	defer tmpSolution.Close()
	defer os.Remove(tmpSolution.Name())
	if err := json.NewEncoder(tmpSolution).Encode(solutionData); err != nil {
		return 0, emptyBonuses, "", err
	}

	dislike, acquiredBonus, rejectReason := eval(tmpProblem.Name(), tmpSolution.Name())
	return dislike, acquiredBonus, rejectReason, nil
}

func eval(problemPath, solutionPath string) (int64, []solutionmgr.UsedBonus, string) {
	cmd := exec.Command(
		*scorerPath,
		problemPath,
		solutionPath,
		"json",
	)
	bs, err := cmd.Output()
	if err != nil {
		return RejectDislike, make([]solutionmgr.UsedBonus, 0), err.Error()
	}
	var output scorerOutput
	if err := json.Unmarshal(bs, &output); err != nil {
		return RejectDislike, make([]solutionmgr.UsedBonus, 0), err.Error()
	}
	if !output.IsValid {
		return RejectDislike, make([]solutionmgr.UsedBonus, 0), "rejected by scorer"
	}
	return output.Dislike, output.AcquiredBonus, ""
}
