package storage

import (
	"fmt"
	"os"
	"path/filepath"
)

type Files struct {
	basePath string
}

func NewFiles(basePath string) (*Files, error) {
	for _, dir := range []string{"problems", "solutions"} {
		if err := os.MkdirAll(filepath.Join(basePath, dir), 0777); err != nil {
			return nil, err
		}
	}
	return &Files{basePath: basePath}, nil
}

func (f *Files) ReadProblem(problemID int64) ([]byte, error) {
	return os.ReadFile(f.problemFilePath(problemID))
}

func (f *Files) ReadSolution(fileHash string) ([]byte, error) {
	return os.ReadFile(f.solutionFilePath(fileHash))
}

func (f *Files) WriteProblem(problemID int64, data []byte) error {
	return os.WriteFile(f.problemFilePath(problemID), data, 0666)
}

func (f *Files) WriteSolution(fileHash string, data []byte) error {
	return os.WriteFile(f.solutionFilePath(fileHash), data, 0666)
}

func (f *Files) problemFilePath(problemID int64) string {
	return filepath.Join(f.basePath, "problems", fmt.Sprintf("%d.json", problemID))
}

func (f *Files) solutionFilePath(fileHash string) string {
	return filepath.Join(f.basePath, "solutions", fmt.Sprintf("%s.json", fileHash))
}
