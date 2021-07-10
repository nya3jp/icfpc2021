#!/bin/bash

cd $(dirname $BASH_SOURCE)
go run cmd/dashboard-server/main.go \
  --persist_path /tmp/dashboard-data \
  --static_path ./front/build \
  --scorer_path ./static/scorer
