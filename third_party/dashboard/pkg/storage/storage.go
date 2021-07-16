// Copyright 2021 Team Special Weekend
// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
