package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"

	"icfpc2021/dashboard/pkg/solutionmgr"

	"github.com/gorilla/mux"
)

var (
	port        = flag.Int("port", 8080, "")
	persistPath = flag.String("persist_path", "/tmp/dashboard-data", "")
	staticPath  = flag.String("static_path", "/tmp/static-data", "")
)

func main() {
	flag.Parse()
	mgr, err := solutionmgr.NewManager(*persistPath)
	if err != nil {
		log.Fatal(err)
	}
	defer mgr.Close()

	s := &server{mgr}
	r := mux.NewRouter()
	r.HandleFunc("/api/solutionsets", s.handleSolutionSetsList).Methods("GET")
	r.HandleFunc("/api/solutionsets/{solution_set}", s.handleSolutionSetsGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solutions/{solution_id}", s.handleProblemsSolutionsGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solutions/{solution_id}/meta", s.handleProblemsSolutionsMetaGet).Methods("GET")
	r.HandleFunc("/api/solutions", s.handleSolutionsList).Methods("GET")
	r.HandleFunc("/api/solutions", s.handleSolutionsPost).Methods("POST")
	r.PathPrefix("/").Handler(http.FileServer(http.Dir(*staticPath)))

	log.Print("Starting...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *port), r))
}

type server struct {
	mgr *solutionmgr.Manager
}

func (s *server) handleSolutionSetsList(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	ret, err := s.mgr.GetRecentSolutionSets(0, 10)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(ret)
}

func (s *server) handleSolutionSetsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	vars := mux.Vars(r)
	ret, err := s.mgr.GetSolutionSet(vars["solution_set"])
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(ret)
}

func (s *server) handleProblemsSolutionsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	vars := mux.Vars(r)
	bs, err := s.mgr.GetSolution(vars["problem_id"], vars["solution_id"])
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if _, err := w.Write(bs); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleProblemsSolutionsMetaGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	vars := mux.Vars(r)
	solution, err := s.mgr.GetSolutionMetadata(vars["problem_id"], vars["solution_id"])
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(solution)
}

func (s *server) handleSolutionsList(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutions, err := s.mgr.GetRecentSolutions(0, 10)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(solutions)
}

func (s *server) handleSolutionsPost(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	if err := r.ParseMultipartForm(32 << 20); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	problemID := r.Form.Get("problem_id")
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
	solutionSet := r.Form.Get("solution_set")
	if err := s.mgr.Add(problemID, solutionJSON, tags, solutionSet); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	io.WriteString(w, "ok")
}
