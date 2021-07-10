package scrape

import (
	"log"
	"time"

	"icfpc2021/dashboard/pkg/solutionmgr"
)

func ScrapeDislikeTask(scraper *Scraper, mgr *solutionmgr.Manager) {
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
