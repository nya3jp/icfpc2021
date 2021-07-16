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

package scrape

import "testing"

func TestScrapeMinimalDislikes(t *testing.T) {
	t.Skip()
	s, err := NewScraper()
	if err != nil {
		t.Fatal(err)
	}
	v, err := s.ScrapeMinimalDislikes()
	if err != nil {
		t.Fatal(err)
	}
	t.Log(v)
}

func TestScrapeSolutions(t *testing.T) {
	t.Skip()
	s, err := NewScraper()
	if err != nil {
		t.Fatal(err)
	}
	v, err := s.ScrapeSolutions(1)
	if err != nil {
		t.Fatal(err)
	}
	t.Log(v)
}

func TestDownloadSolution(t *testing.T) {
	t.Skip()
	s, err := NewScraper()
	if err != nil {
		t.Fatal(err)
	}
	v, err := s.DownloadSolution("c527c1b8-73d9-4ec8-8f66-c9a19e6fab20")
	if err != nil {
		t.Fatal(err)
	}
	t.Log(v)
}
