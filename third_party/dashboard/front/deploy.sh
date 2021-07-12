#!/bin/bash

set -ex

docker build -t gcr.io/special-weekend-2021/ui-server:latest .
docker push gcr.io/special-weekend-2021/ui-server
kubectl rollout restart deployment ui-server-deployment
