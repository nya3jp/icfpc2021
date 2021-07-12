package main

import (
	"context"
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
	"time"

	"icfpc2021/dashboard/pkg/eval"
	"icfpc2021/dashboard/pkg/scrape"
	"icfpc2021/dashboard/pkg/solutionmgr"
	"icfpc2021/dashboard/pkg/tasks"

	"github.com/golang/protobuf/jsonpb"
	"github.com/gorilla/mux"
	"github.com/nya3jp/flex"
	"github.com/nya3jp/flex/flexpb"
	"google.golang.org/protobuf/proto"
)

var (
	port         = flag.Int("port", 8080, "")
	uiServer     = flag.String("ui_server", "", "")
	enableScrape = flag.Bool("enable_scrape", true, "")
)

func main() {
	flag.Parse()
	mgr, err := solutionmgr.NewMySQLManager()
	if err != nil {
		log.Fatal(err)
	}
	defer mgr.Close()

	flexClient, err := tasks.NewFlexClient()
	if err != nil {
		log.Fatal(err)
	}

	scraper, err := scrape.NewScraper()
	if err != nil {
		log.Printf("Cannot create a scraper. Disable scraping part: %v", err)
	} else {
		if *enableScrape {
			go scrape.ScrapeSubmittedSolutionsTask(scraper, mgr)
			go scrape.ScrapeDislikeTask(scraper, mgr)
		}
	}

	u, err := url.Parse(*uiServer)
	if err != nil {
		log.Fatal(err)
	}
	uiHandler := httputil.NewSingleHostReverseProxy(u)

	s := &server{mgr, scraper, flexClient}
	r := mux.NewRouter()
	r.HandleFunc("/api/problems", s.handleProblemsGet).Methods("GET")
	r.HandleFunc("/api/problems", s.handleProblemsPost).Methods("POST")
	r.HandleFunc("/api/problems/{problem_id}", s.handleProblemGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solve", s.handleProblemSolve).Methods("POST")
	r.HandleFunc("/api/problems/{problem_id}/solutions", s.handleProblemSolutionsGet).Methods("GET")
	r.HandleFunc("/api/problems/{problem_id}/solutions", s.handleProblemSolutionsPost).Methods("POST")
	r.HandleFunc("/api/solutions/{solution_id}", s.handleSolutionGet).Methods("GET")
	r.HandleFunc("/api/solutions/{solution_id}/submit", s.handleSolutionSubmit).Methods("POST")
	r.HandleFunc("/api/solutions/{solution_id}/tags", s.handleSolutionAddTag).Methods("POST")
	r.HandleFunc("/api/solutions/{solution_id}/tags", s.handleSolutionDeleteTag).Methods("DELETE")
	r.HandleFunc("/api/solutions", s.handleSolutionsGet).Methods("GET")
	r.HandleFunc("/api/solutions", s.handleSolutionsPost).Methods("POST")
	r.HandleFunc("/api/submittedsolutions", s.handleSubmittedSolutionsGet).Methods("GET")
	r.HandleFunc("/api/tasks/running", s.handleTasksRunningGet).Methods("GET")
	r.HandleFunc("/api/tasks/info/{task_id}", s.handleTaskGet).Methods("GET")
	r.HandleFunc("/api/tasks/add", s.handleAddTaskPost).Methods("POST")
	r.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		io.WriteString(w, "ok")
	})
	r.PathPrefix("/").Handler(uiHandler)

	log.Print("Starting...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *port), r))
}

type server struct {
	mgr        *solutionmgr.MySQLManager
	scraper    *scrape.Scraper
	flexClient *flex.Client
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

func (s *server) handleProblemSolve(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	problemID, err := strconv.ParseInt(mux.Vars(r)["problem_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	bonus := r.URL.Query().Get("bonus")
	deadlineSec, err := strconv.ParseInt(r.URL.Query().Get("deadline"), 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	timeLimitSec, err := strconv.ParseInt(r.URL.Query().Get("time_limit"), 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	penaltyRatio, err := strconv.ParseInt(r.URL.Query().Get("penalty_ratio"), 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	// TODO: Parameter validation

	if _, err := s.mgr.GetProblem(problemID); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	taskSpec := tasks.NewTask(time.Duration(deadlineSec)*time.Second, time.Duration(timeLimitSec)*time.Second, penaltyRatio, problemID, bonus)
	taskID, err := s.flexClient.AddTask(r.Context(), taskSpec)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if err := s.mgr.AddRunningTask(taskID, problemID,); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if err := json.NewEncoder(w).Encode(taskID); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleTasksRunningGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")

	tasks, err := s.mgr.GetRunningTasks()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(tasks); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleTaskGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	taskID, err := strconv.ParseInt(mux.Vars(r)["task_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	task, err := s.flexClient.GetTask(context.Background(), taskID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	if err := (&jsonpb.Marshaler{}).Marshal(w, task); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func (s *server) handleAddTaskPost(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	var spec flexpb.TaskSpec
	if err := proto.Unmarshal(body, &spec); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// Spot check
	if spec.GetCommand().GetShell() == "" {
		http.Error(w, "Invalid TaskSpec", http.StatusInternalServerError)
		return
	}

	id, err := s.flexClient.AddTask(context.Background(), &spec)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	fmt.Fprint(w, id)
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

func (s *server) handleSolutionsGet(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	tag := r.URL.Query().Get("tag")
	if tag == "" {
		http.Error(w, "tag is needed", http.StatusBadRequest)
		return
	}
	solutions, err := s.mgr.GetSolutionsForTag(tag)
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

func (s *server) handleSolutionAddTag(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutionID, err := strconv.ParseInt(mux.Vars(r)["solution_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	tag := r.URL.Query().Get("tag")
	if tag == "" {
		http.Error(w, "tag is needed", http.StatusBadRequest)
		return
	}
	if err := s.mgr.AddSolutionTag(solutionID, tag); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
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

func (s *server) handleSolutionDeleteTag(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	solutionID, err := strconv.ParseInt(mux.Vars(r)["solution_id"], 10, 64)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	tag := r.URL.Query().Get("tag")
	if tag == "" {
		http.Error(w, "tag is needed", http.StatusBadRequest)
		return
	}
	if err := s.mgr.RemoveSolutionTag(solutionID, tag); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
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
