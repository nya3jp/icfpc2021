package solutionmgr

import "log"

func BatchDedupe(m *Manager) error {
	problems, err := m.GetProblems()
	if err != nil {
		return err
	}
	for _, problem := range problems {
		solutions, err := m.GetSolutionsForProblem(problem.ProblemID)
		if err != nil {
			return err
		}
		idm := map[string][]int64{}
		for _, solution := range solutions {
			idm[solution.fileHash] = append(idm[solution.fileHash], solution.SolutionID)
		}
		for _, ids := range idm {
			if len(ids) == 1 {
				continue
			}
			for _, id := range ids[1:] {
				log.Printf("Delete %d", id)
				if err := m.deleteSolution(id); err != nil {
					return err
				}
			}
		}
	}
	return nil
}

func BatchRemoveEmptyTags(m *Manager) error {
	_, err := m.db.Exec("DELETE FROM tags WHERE tag = ?", "")
	if err != nil {
		return err
	}
	return nil
}
