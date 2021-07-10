package main

import (
	"encoding/json"
	"flag"
	"io/ioutil"
	"log"
	"strings"

	"github.com/gocolly/colly"
)

const (
	baseUrl     = "https://poses.live/"
	loginUrl    = baseUrl + "login"
	problemsUrl = baseUrl + "problems"
)

var (
	email    = flag.String("email", "", "")
	password = flag.String("password", "", "")
)

type Problem struct {
	ProblemID string   `json:"problem_id"`
	Solutions []string `json:"solutions"`
}

func main() {
	flag.Parse()

	login := colly.NewCollector()
	problemList := login.Clone()
	solutions := login.Clone()
	solution := login.Clone()

	problems := make([]Problem, 0)

	err := login.Post(loginUrl, map[string]string{
		"login.email":    *email,
		"login.password": *password,
	})
	if err != nil {
		log.Fatal(err)
	}
	login.OnResponse(func(r *colly.Response) {
		log.Println("response received", r.StatusCode)
		problemList.Visit(problemsUrl)
		problemList.Wait()
	})

	problemList.OnHTML("a[href]", func(e *colly.HTMLElement) {
		link := e.Attr("href")
		if !strings.HasPrefix(link, "/problems/") {
			return
		}
		p := Problem{
			ProblemID: e.Text,
			Solutions: make([]string, 0),
		}
		problems = append(problems, p)
		var problemUrl = baseUrl + link
		solutions.Visit(problemUrl)
		solutions.Wait()
	})

	solutions.OnHTML("a[href]", func(e *colly.HTMLElement) {
		link := e.Attr("href")
		if !strings.HasPrefix(link, "/solutions") {
			return
		}

		var solutionUrl = baseUrl + link + "/download"
		log.Println(solutionUrl)
		solution.Visit(solutionUrl)
		solution.Wait()
	})

	solution.OnResponse(func(r *colly.Response) {
		problems[len(problems)-1].Solutions = append(problems[len(problems)-1].Solutions, string(r.Body))
	})

	login.Visit(loginUrl)
	login.Wait()

	res, err := json.Marshal(problems)
	if err != nil {
		log.Fatalf("Failed to marshal data: %q", problems)
	}
	ioutil.WriteFile("solutions.json", res, 0666)
}
