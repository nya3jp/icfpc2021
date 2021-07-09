package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"html"
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
	r.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello, %q", html.EscapeString(r.URL.Path))
	})
	r.HandleFunc("/api/solutions", s.handleSolutionsGet).Methods("GET")
	r.HandleFunc("/api/solutions", s.handleSolutionsPost).Methods("POST")

	log.Print("Starting...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *port), r))
}

type server struct {
	mgr *solutionmgr.Manager
}

func (s *server) handleSolutionsGet(w http.ResponseWriter, r *http.Request) {
	solutions, err := s.mgr.GetRecentSolutions(0, 10)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(solutions)
}

func (s *server) handleSolutionsPost(w http.ResponseWriter, r *http.Request) {
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
