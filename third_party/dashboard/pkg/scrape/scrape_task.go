// Copyright 2021 Team Special Weekend
// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
			dislike, acquiredBonus, rejectReason, err := eval.EvalData(&problem.Data, data)
			if err != nil {
				return err
			}
			sol := &solutionmgr.Solution{
				ProblemID:     problem.ProblemID,
				CreatedAt:     subsol.CreatedAt,
				Dislike:       dislike,
				RejectReason:  rejectReason,
				AcquiredBonus: acquiredBonus,
				Data:          *data,
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
