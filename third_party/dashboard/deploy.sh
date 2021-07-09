#!/bin/bash

set -ex

docker build -t gcr.io/special-weekend-2021/dashboard-server:latest .
docker push gcr.io/special-weekend-2021/dashboard-server
kubectl rollout restart deployment dashboard-server-deployment
