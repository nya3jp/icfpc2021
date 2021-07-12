package main


import (
	"bytes"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/nya3jp/flex/flexpb"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"
)

func main() {
	if err := func() error {
		server := flag.String("server", "https://spweek.badalloc.com", "API server URL")
		command := flag.String("command", "", "Command to run")
		pkg := flag.String("package", "", "URL of package to use")
		priority := flag.Int64("priority", 0, "Priority (higher comes first)")
		timeLimit := flag.Int64("timelimit", 0, "Time limit in seconds")
		flag.Parse()

		if *command == "" {
			return errors.New("-command is missing")
		}
		if *timeLimit == 0 {
			return errors.New("-timelimit is missing")
		}
		if *pkg == "" {
			return errors.New("-pkg is missing")
		}

		spec := &flexpb.TaskSpec{
			Command: &flexpb.TaskCommand{
				Shell: *command,
			},
			Constraints: &flexpb.TaskConstraints{
				Priority: int32(*priority),
			},
			Limits:   &flexpb.TaskLimits{Time: durationpb.New(time.Second * time.Duration(*timeLimit))},
			Packages: []*flexpb.TaskPackage{{Url: *pkg}},
		}
		req, err := proto.Marshal(spec)
		if err != nil {
			return err
		}

		res, err := http.Post(*server + "/api/tasks/add", "application/protocol-buffers", bytes.NewBuffer(req))
		if err != nil {
			return err
		}
		defer res.Body.Close()

		if res.StatusCode != http.StatusOK {
			io.Copy(os.Stderr, res.Body)
			return fmt.Errorf("server responded with error: %s", res.Status)
		}

		b, err := io.ReadAll(res.Body)
		fmt.Printf("OK: %s/#/tasks/%s\n", *server, string(b))
		return nil
	}(); err != nil {
		log.Fatalf("ERROR: %v", err)
	}
}
