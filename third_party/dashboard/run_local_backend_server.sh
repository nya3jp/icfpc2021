#!/bin/bash

cd $(dirname $BASH_SOURCE)
go run cmd/dashboard-server/main.go \
  --scorer_path ./static/scorer \
  "$@"
