package scrape

import (
	"log"
	"time"

	"icfpc2021/dashboard/pkg/eval"
	"icfpc2021/dashboard/pkg/solutionmgr"
)

func ScrapeDislikeTask(scraper *Scraper, mgr *solutionmgr.MySQLManager) {
	tick := time.NewTicker(5 * time.Minute)
	defer tick.Stop()
	for {
		select {
		case <-tick.C:
			m, err := scraper.ScrapeMinimalDislikes()
			if err != nil {
				log.Printf("Failed to scrape dislikes: %v", err)
				continue
			}
			for problemID, minimalDislike := range m {
				if err := mgr.UpdateMinimalDislike(problemID, minimalDislike); err != nil {
					log.Printf("Failed to update the minimal dislike: %v", err)
				}
			}
		}
	}
}

func ScrapeSubmittedSolutionsTask(scraper *Scraper, m *solutionmgr.MySQLManager) {
	if err := scrapeSubmittedSolutionsTaskOnce(scraper, m); err != nil {
		log.Printf("Failed to scrape the submitted solutions: %v", err)
	}

	tick := time.NewTicker(10 * time.Minute)
	defer tick.Stop()
	for {
		select {
		case <-tick.C:
			if err := scrapeSubmittedSolutionsTaskOnce(scraper, m); err != nil {
				log.Printf("Failed to scrape the submitted solutions: %v", err)
			}
		}
	}
}

func scrapeSubmittedSolutionsTaskOnce(scraper *Scraper, m *solutionmgr.MySQLManager) error {
	solutions, err := m.GetSubmittedSolutions()
	if err != nil {
		return err
	}
	alreadyExist := map[string]bool{}
	for _, s := range solutions {
		alreadyExist[s.SubmittedSolutionID] = true
	}

	problems, err := m.GetProblems()
	if err != nil {
		return err
	}
	for _, problem := range problems {
		ss, err := scraper.ScrapeSolutions(problem.ProblemID)
		if err != nil {
			return err
		}
		for _, subsol := range ss {
			if alreadyExist[subsol.SubmittedSolutionID] {
				continue
			}
			data, err := scraper.DownloadSolution(subsol.SubmittedSolutionID)
			if err != nil {
				return err
			}
			dislike, rejectReason, err := eval.EvalData(&problem.Data, data)
			if err != nil {
				return err
			}
			sol := &solutionmgr.Solution{
				ProblemID:    problem.ProblemID,
				CreatedAt:    subsol.CreatedAt,
				Dislike:      dislike,
				RejectReason: rejectReason,
				Data:         *data,
			}
			solutionID, err := m.AddSolution(sol)
			if err != nil {
				return err
			}
			subsol.SolutionID = solutionID
			if err := m.AddSubmittedSolution(subsol); err != nil {
				return err
			}
		}
	}
	return nil
}
