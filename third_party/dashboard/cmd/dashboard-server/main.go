package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
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
	r.HandleFunc("/api/solutions/{solution_id}", s.handleSolutionsGet).Methods("GET")
	r.HandleFunc("/api/solutions/{solution_id}/meta", s.handleSolutionsMetaGet).Methods("GET")
	r.HandleFunc("/api/solutions", s.handleSolutionsPost).Methods("POST")
	r.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		io.WriteString(w, "ok")
	})
	r.PathPrefix("/").Handler(http.FileServer(http.Dir(*staticPath)))

	log.Print("Starting...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *port), r))
}

type server struct {
	mgr *solutionmgr.Manager
}

func (s *server) handleSolutionsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutionID, err := strconv.ParseInt(mux.Vars(r)["solution_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	bs, err := s.mgr.GetSolution(solutionID)
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

func (s *server) handleSolutionsMetaGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutionID, err := strconv.ParseInt(mux.Vars(r)["solution_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	solution, err := s.mgr.GetSolutionMetadata(solutionID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(solution)
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
	if err := s.mgr.AddSolution(problemID, solutionJSON, tags); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	io.WriteString(w, "ok")
}
