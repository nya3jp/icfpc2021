package scrape

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"time"

	"icfpc2021/dashboard/pkg/solutionmgr"

	"github.com/gocolly/colly"
)

const (
	baseURL      = "https://poses.live/"
	loginURL     = baseURL + "login"
	problemsURL  = baseURL + "problems"
	solutionsURL = baseURL + "solutions"
)

const (
	ICFPC_API_KEY_KEY  = "ICFPC_API_KEY"
	ICFPC_EMAIL_KEY    = "ICFPC_EMAIL"
	ICFPC_PASSWORD_KEY = "ICFPC_PASSWORD"
)

type Scraper struct {
	login    *colly.Collector
	apiKey   string
	email    string
	password string
}

func NewScraper() (*Scraper, error) {
	apiKey, ok := os.LookupEnv(ICFPC_API_KEY_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the ICFPC API key")
	}
	email, ok := os.LookupEnv(ICFPC_EMAIL_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the ICFPC email")
	}
	password, ok := os.LookupEnv(ICFPC_PASSWORD_KEY)
	if !ok {
		return nil, fmt.Errorf("cannot find the ICFPC password")
	}
	return &Scraper{
		apiKey:   apiKey,
		email:    email,
		password: password,
	}, nil
}

func (s *Scraper) loginSession() (*colly.Collector, error) {
	if s.login != nil {
		// Looks like the credential has a long expiration time. Reuse
		// as much as possible.
		return s.login, nil
	}
	login := colly.NewCollector()
	err := login.Post(loginURL, map[string]string{
		"login.email":    s.email,
		"login.password": s.password,
	})
	if err != nil {
		return nil, err
	}
	login.Visit(loginURL)
	login.Wait()
	return login, nil
}

func (s *Scraper) ScrapeMinimalDislikes() (map[int64]int64, error) {
	login, err := s.loginSession()
	if err != nil {
		return nil, err
	}

	problemList := login.Clone()
	m := map[int64]int64{}
	problemList.OnHTML("tr", func(e *colly.HTMLElement) {
		first := e.DOM.Children().First().Text()
		last := e.DOM.Children().Last().Text()
		problemID, err := strconv.ParseInt(first, 10, 64)
		if err != nil {
			// Skip this. Likely the header.
			return
		}
		minimalDislike, err := strconv.ParseInt(last, 10, 64)
		if err != nil {
			return
		}
		m[problemID] = minimalDislike
	})
	problemList.Visit(problemsURL)
	problemList.Wait()
	return m, nil
}

func (s *Scraper) ScrapeSolutions(problemID int64) ([]*solutionmgr.SubmittedSolution, error) {
	log.Printf("Scraping solutions for %d", problemID)
	login, err := s.loginSession()
	if err != nil {
		return nil, err
	}

	problemPage := login.Clone()
	var ret []*solutionmgr.SubmittedSolution
	problemPage.OnHTML("tr", func(e *colly.HTMLElement) {
		if !e.DOM.Children().First().Is("td") {
			return
		}
		link, _ := e.DOM.Children().First().Children().First().Attr("href")
		if !strings.HasPrefix(link, "/solutions") {
			return
		}
		dislikeStr := e.DOM.Children().Last().Text()
		if _, err := strconv.ParseInt(dislikeStr, 10, 64); err != nil {
			return
		}
		createdAt, err := time.Parse(time.RFC3339Nano, e.DOM.Children().First().Children().First().Text())
		if err != nil {
			log.Printf("Cannot parse the creation time: %v", err)
			return
		}
		ret = append(ret, &solutionmgr.SubmittedSolution{
			ProblemID:           problemID,
			SubmittedSolutionID: strings.TrimSuffix(strings.TrimPrefix(link, "/solutions/"), "/download"),
			CreatedAt:           createdAt.Unix(),
		})
	})
	problemPage.Visit(problemsURL + fmt.Sprintf("/%d", problemID))
	problemPage.Wait()
	return ret, nil
}

func (s *Scraper) DownloadSolution(submittedSolutionID string) (*solutionmgr.SolutionData, error) {
	log.Printf("Downloading solution %s", submittedSolutionID)
	login, err := s.loginSession()
	if err != nil {
		return nil, err
	}

	solutionPage := login.Clone()
	var ret solutionmgr.SolutionData
	solutionPage.OnResponse(func(r *colly.Response) {
		log.Print(r.StatusCode)
		err := json.Unmarshal(r.Body, &ret)
		if err != nil {
			log.Printf("Cannot unmarshal submitted solution (%s): %v", string(r.Body), err)
			return
		}
	})
	solutionPage.Visit(solutionsURL + fmt.Sprintf("/%s/download", submittedSolutionID))
	solutionPage.Wait()
	return &ret, nil
}
