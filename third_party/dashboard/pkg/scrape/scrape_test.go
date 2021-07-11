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
