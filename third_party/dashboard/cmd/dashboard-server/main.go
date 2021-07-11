package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strconv"
	"strings"

	"icfpc2021/dashboard/pkg/eval"
	"icfpc2021/dashboard/pkg/scrape"
	"icfpc2021/dashboard/pkg/solutionmgr"

	"github.com/gorilla/mux"
)

var (
	port         = flag.Int("port", 8080, "")
	persistPath  = flag.String("persist_path", "/tmp/dashboard-data", "")
	staticPath   = flag.String("static_path", "/tmp/static-data", "")
	uiServer     = flag.String("ui_server", "", "")
	enableScrape = flag.Bool("enable_scrape", true, "")
)

func newManager() solutionmgr.Manager {
	sqlite, err := solutionmgr.NewSQLiteManager(*persistPath)
	if err != nil {
		log.Fatal(err)
	}
	defer sqlite.Close()

	mysql, err := solutionmgr.NewMySQLManager()
	if err != nil {
		log.Fatal(err)
	}

	problems, err := sqlite.GetProblems()
	if err != nil {
		log.Fatal(err)
	}
	for _, problem := range problems {
		log.Printf("Adding a problem %d", problem.ProblemID)
		if err := mysql.AddProblem(problem); err != nil {
			log.Fatal(err)
		}

		solutions, err := sqlite.GetSolutionsForProblem(problem.ProblemID)
		if err != nil {
			log.Fatal(err)
		}
		for _, solution := range solutions {
			log.Printf("Adding a solution %d", solution.SolutionID)
			if _, err := mysql.AddSolution(solution); err != nil {
				log.Fatal(err)
			}
		}
	}
	ssolutions, err := sqlite.GetSubmittedSolutions()
	if err != nil {
		log.Fatal(err)
	}
	for _, ssolution := range ssolutions {
		log.Printf("Adding a submitted solution %s", ssolution.SubmittedSolutionID)
		if err := mysql.AddSubmittedSolution(ssolution); err != nil {
			log.Fatal(err)
		}
	}
	if err := mysql.SetSolutionAutoIncrement(); err != nil {
		log.Fatal(err)
	}

	return mysql
}

func main() {
	flag.Parse()
	mgr := newManager()
	defer mgr.Close()

	scraper, err := scrape.NewScraper()
	if err != nil {
		log.Printf("Cannot create a scraper. Disable scraping part: %v", err)
	} else {
		if *enableScrape {
			go scrape.ScrapeSubmittedSolutionsTask(scraper, mgr)
			go scrape.ScrapeDislikeTask(scraper, mgr)
		}
	}

	var fallbackHandler http.Handler
	if *uiServer != "" {
		u, err := url.Parse(*uiServer)
		if err != nil {
			log.Fatal(err)
		}
		fallbackHandler = httputil.NewSingleHostReverseProxy(u)
	} else {
		fallbackHandler = http.FileServer(http.Dir(*staticPath))
	}

	s := &server{mgr, scraper}
	r := mux.NewRouter()
	r.HandleFunc("/api/problems", s.handleProblemsGet).Methods("GET")
	r.HandleFunc("/api/problems", s.handleProblemsPost).Methods("POST")
	r.HandleFunc("/api/problems/{problem_id}", s.handleProblemGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solutions", s.handleProblemSolutionsGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solutions", s.handleProblemSolutionsPost).Methods("POST")
	r.HandleFunc("/api/solutions/{solution_id}", s.handleSolutionGet).Methods("GET")
	r.HandleFunc("/api/solutions/{solution_id}/submit", s.handleSolutionSubmit).Methods("POST")
	r.HandleFunc("/api/solutions", s.handleSolutionsPost).Methods("POST")
	r.HandleFunc("/api/submittedsolutions", s.handleSubmittedSolutionsGet).Methods("GET")
	r.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		io.WriteString(w, "ok")
	})
	r.PathPrefix("/").Handler(fallbackHandler)

	log.Print("Starting...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *port), r))
}

type server struct {
	mgr     solutionmgr.Manager
	scraper *scrape.Scraper
}

func (s *server) handleProblemsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	problems, err := s.mgr.GetProblems()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(problems); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleProblemGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	problemID, err := strconv.ParseInt(mux.Vars(r)["problem_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	problem, err := s.mgr.GetProblem(problemID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(problem); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleProblemSolutionsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	problemID, err := strconv.ParseInt(mux.Vars(r)["problem_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	solutions, err := s.mgr.GetSolutionsForProblem(problemID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(solutions); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleSubmittedSolutionsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutions, err := s.mgr.GetSubmittedSolutions()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(solutions); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleSolutionGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutionID, err := strconv.ParseInt(mux.Vars(r)["solution_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	solution, err := s.mgr.GetSolution(solutionID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(solution); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleProblemsPost(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	if err := r.ParseMultipartForm(32 << 20); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	problemID, err := strconv.ParseInt(r.Form.Get("problem_id"), 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	file, _, err := r.FormFile("problem")
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	defer file.Close()
	b, err := io.ReadAll(file)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	var data solutionmgr.ProblemData
	if err := json.Unmarshal(b, &data); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	problem := &solutionmgr.Problem{
		ProblemID:      problemID,
		MinimalDislike: eval.RejectDislike,
		Data:           data,
	}
	if err := s.mgr.AddProblem(problem); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	io.WriteString(w, "ok")
}

func (s *server) handleProblemSolutionsPost(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	problemID, err := strconv.ParseInt(mux.Vars(r)["problem_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	b, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	var data solutionmgr.SolutionData
	if err := json.Unmarshal(b, &data); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	problem, err := s.mgr.GetProblem(problemID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	dislike, rejectReason, err := eval.EvalData(&problem.Data, &data)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	solution := &solutionmgr.Solution{
		ProblemID:    problemID,
		Dislike:      dislike,
		RejectReason: rejectReason,
		Data:         data,
	}
	solutionID, err := s.mgr.AddSolution(solution)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	solution, err = s.mgr.GetSolution(solutionID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(solution); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleSolutionsPost(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	if err := r.ParseMultipartForm(32 << 20); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	problemID, err := strconv.ParseInt(r.Form.Get("problem_id"), 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	file, _, err := r.FormFile("solution")
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	defer file.Close()
	b, err := io.ReadAll(file)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	var data solutionmgr.SolutionData
	if err := json.Unmarshal(b, &data); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	tags := trimAndRemoveEmpty(strings.Split(r.Form.Get("tags"), ","))
	problem, err := s.mgr.GetProblem(problemID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	dislike, rejectReason, err := eval.EvalData(&problem.Data, &data)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	solution := &solutionmgr.Solution{
		ProblemID:    problemID,
		Tags:         tags,
		Dislike:      dislike,
		RejectReason: rejectReason,
		Data:         data,
	}
	solutionID, err := s.mgr.AddSolution(solution)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	solution, err = s.mgr.GetSolution(solutionID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(solution); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleSolutionSubmit(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	if s.scraper == nil {
		http.Error(w, "do not have an ICFPC credential", http.StatusInternalServerError)
		return
	}
	solutionID, err := strconv.ParseInt(mux.Vars(r)["solution_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	solution, err := s.mgr.GetSolution(solutionID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	submitID, err := s.scraper.SubmitSolution(solution.ProblemID, &solution.Data)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	io.WriteString(w, submitID)
}

func trimAndRemoveEmpty(ss []string) []string {
	var res []string
	for _, s := range ss {
		s = strings.TrimSpace(s)
		if s != "" {
			res = append(res, s)
		}
	}
	return res
}
