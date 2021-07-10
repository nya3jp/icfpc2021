package main

import (
	"encoding/json"
	"flag"
	"io/ioutil"
	"log"
	"os"
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
	file     = flag.String("solution_file", "solutions.json", "")
)

type Problem struct {
	ProblemID string `json:"problem_id"`
	Solutions []Pose `json:"solutions"`
}

type Pose struct {
	Vertices [][]int `json:"vertices"`
	Name     string  `json:"name"`
}

func contains(problems []Problem, problemId string) bool {
	for _, p := range problems {
		if p.ProblemID == problemId {
			return true
		}
	}
	return false
}

func findProblem(problems []Problem, problemId string) *Problem {
	for _, p := range problems {
		if p.ProblemID == problemId {
			return &p
		}
	}
	return nil
}

func containsSolution(solutions []Pose, solutionId string) bool {
	for _, s := range solutions {
		if s.Name == solutionId {
			return true
		}
	}
	return false
}

func main() {
	flag.Parse()

	login := colly.NewCollector()
	problemList := login.Clone()
	solutions := login.Clone()
	solution := login.Clone()

	problems := make([]Problem, 0)

	_, err := os.Stat(*file)
	if err != nil {
		res, err := json.Marshal(problems)
		if err != nil {
			log.Fatal(err)
		}
		ioutil.WriteFile(*file, res, 0666)
	}

	raw, err := ioutil.ReadFile(*file)
	if err != nil {
		log.Fatal(err)
	}

	err = json.Unmarshal(raw, &problems)
	if err != nil {
		log.Fatal(err)
	}

	err = login.Post(loginUrl, map[string]string{
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

	currentProblemId := "0"
	problemList.OnHTML("a[href]", func(e *colly.HTMLElement) {
		link := e.Attr("href")
		if !strings.HasPrefix(link, "/problems/") {
			return
		}

		currentProblemId = e.Text
		if !contains(problems, currentProblemId) {
			p := Problem{
				ProblemID: currentProblemId,
				Solutions: make([]Pose, 0),
			}
			problems = append(problems, p)
		}
		var problemUrl = baseUrl + link
		solutions.Visit(problemUrl)
		solutions.Wait()
	})

	solutions.OnHTML("a[href]", func(e *colly.HTMLElement) {
		link := e.Attr("href")
		if !strings.HasPrefix(link, "/solutions") {
			return
		}

		var solutionName = strings.TrimSuffix(strings.TrimPrefix(link, "/solutions"), "/download")
		for _, p := range problems {
			if p.ProblemID == currentProblemId {
				for _, s := range p.Solutions {
					if s.Name == solutionName {
						return
					}
				}
			}
		}

		var solutionUrl = baseUrl + link + "/download"
		log.Println(solutionUrl)
		solution.Visit(solutionUrl)
		solution.Wait()
	})

	solution.OnResponse(func(r *colly.Response) {
		var p Pose
		err := json.Unmarshal(r.Body, &p)
		if len(r.Body) == 0 {
			return
		}
		if err != nil {
			log.Printf("Failed to unmarshal response: %s %q", r.Body, err)
			return
		}
		p.Name = strings.TrimSuffix(r.FileName(), ".solution")

		log.Printf("%s\n", r.Request.URL.Path)
		for i, _ := range problems {
			if problems[i].ProblemID == currentProblemId {
				var found = false
				for j, _ := range problems[i].Solutions {
					if problems[i].Solutions[j].Name == p.Name {
						found = true
					}
				}
				if !found {
					problems[i].Solutions = append(problems[i].Solutions, p)
				}
			}
		}
	})

	login.Visit(loginUrl)
	login.Wait()

	res, err := json.Marshal(problems)
	if err != nil {
		log.Fatalf("Failed to marshal data: %q", problems)
	}
	ioutil.WriteFile(*file, res, 0666)
}
