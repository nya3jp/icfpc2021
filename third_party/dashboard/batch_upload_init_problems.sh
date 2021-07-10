#!/bin/bash

cd $(dirname $BASH_SOURCE)
cd ../problems
for f in *; do
  problem_id="${f%.*}"
  curl -F problem_id=$problem_id -F problem=@$f http://localhost:8080/api/problems
done
