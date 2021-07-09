package solutionmgr

// import (
// 	"context"
// 	"fmt"
// 	"log"
// 	"os"
// 
// 	"github.com/go-git/go-git/config"
// 	"github.com/go-git/go-git/plumbing"
// 	"github.com/go-git/go-git/plumbing/transport/ssh"
// 	"github.com/go-git/go-git/storage/memory"
// )
// 
// func scanGitRepo(ctx context.Context) error {
// 	pk, err := ssh.NewPublicKeysFromFile("git", *privateKeyPath, "")
// 	if err != nil {
// 		return fmt.Errorf("cannot parse the private key: %v", err)
// 	}
// 
// 	r, err := git.Init(memory.NewStorage(), nil)
// 	if err != nil {
// 		return fmt.Errorf("cannot create a a repository: %v", err)
// 	}
// 
// 	remote := git.NewRemote(r.Storer, &config.RemoteConfig{
// 		Name: "origin",
// 		URLs: []string{"git@github.com:nya3jp/icfpc2021.git"},
// 		Fetch: []config.RefSpec{
// 			config.RefSpec("+refs/solutions/*:refs/solutions/*"),
// 		},
// 	})
// 	if err := remote.FetchContext(ctx, &git.FetchOptions{
// 		Auth:     pk,
// 		Progress: os.Stderr,
// 	}); err != nil {
// 		if err.Error() != "already up-to-date" {
// 			return fmt.Errorf("cannot fetch the repository: %v", err)
// 		}
// 	}
// 
// 	iter, err := r.References()
// 	if err != nil {
// 		return fmt.Errorf("cannot list references: %v", err)
// 	}
// 	if err := iter.ForEach(func(ref *plumbing.Reference) error {
// 		if ref.Type() != plumbing.HashReference {
// 			return nil
// 		}
// 		log.Print(ref)
// 		return nil
// 	}); err != nil {
// 		return err
// 	}
// 	return nil
// }
