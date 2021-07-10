package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"

	"icfpc2021/dashboard/pkg/eval"
	"icfpc2021/dashboard/pkg/solutionmgr"

	"github.com/gorilla/mux"
)

var (
	port        = flag.Int("port", 8080, "")
	persistPath = flag.String("persist_path", "/tmp/dashboard-data", "")
	staticPath  = flag.String("static_path", "/tmp/static-data", "")
	scorerPath  = flag.String("scorer_path", "/static/scorer", "")
)

func main() {
	flag.Parse()
	mgr, err := solutionmgr.NewManager(*persistPath)
	if err != nil {
		log.Fatal(err)
	}
	defer mgr.Close()

	updateDislike := make(chan bool, 1)
	go eval.UpdateDislikeTask(context.Background(), *scorerPath, mgr, updateDislike)

	s := &server{mgr, updateDislike}
	r := mux.NewRouter()
	r.HandleFunc("/api/problems", s.handleProblemsGet).Methods("GET")
	r.HandleFunc("/api/problems", s.handleProblemsPost).Methods("POST")
	r.HandleFunc("/api/problems/{problem_id}", s.handleProblemGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solutions", s.handleProblemSolutionsGet).Methods("GET")
	r.HandleFunc("/api/solutions/{solution_id}", s.handleSolutionGet).Methods("GET")
	r.HandleFunc("/api/solutions", s.handleSolutionsPost).Methods("POST")
	r.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		io.WriteString(w, "ok")
	})
	r.PathPrefix("/").Handler(http.FileServer(http.Dir(*staticPath)))

	log.Print("Starting...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *port), r))
}

type server struct {
	mgr           *solutionmgr.Manager
	updateDislike chan<- bool
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
	problemJSON, err := io.ReadAll(file)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	problem := &solutionmgr.Problem{
		ProblemID: problemID,
		Data:      problemJSON,
	}
	if err := s.mgr.AddProblem(problem); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	io.WriteString(w, "ok")
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
	solutionJSON, err := io.ReadAll(file)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	tags := strings.Split(r.Form.Get("tags"), ",")
	solution := &solutionmgr.Solution{
		ProblemID: problemID,
		Tags:      tags,
		Data:      solutionJSON,
	}
	if err := s.mgr.AddSolution(solution); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	select {
	case s.updateDislike <- true:
	default:
	}
	io.WriteString(w, "ok")
}
