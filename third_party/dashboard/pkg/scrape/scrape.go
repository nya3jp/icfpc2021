package scrape

import (
	"fmt"
	"os"
	"strconv"

	"github.com/gocolly/colly"
)

const (
	baseUrl     = "https://poses.live/"
	loginUrl    = baseUrl + "login"
	problemsUrl = baseUrl + "problems"
)

const (
	ICFPC_API_KEY_KEY  = "ICFPC_API_KEY"
	ICFPC_EMAIL_KEY    = "ICFPC_EMAIL"
	ICFPC_PASSWORD_KEY = "ICFPC_PASSWORD"
)

type Scraper struct {
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

func (s *Scraper) ScrapeMinimalDislikes() (map[int64]int64, error) {
	login := colly.NewCollector()
	problemList := login.Clone()
	err := login.Post(loginUrl, map[string]string{
		"login.email":    s.email,
		"login.password": s.password,
	})
	if err != nil {
		return nil, err
	}
	login.OnResponse(func(r *colly.Response) {
		problemList.Visit(problemsUrl)
		problemList.Wait()
	})
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
	login.Visit(loginUrl)
	login.Wait()
	return m, nil
}
