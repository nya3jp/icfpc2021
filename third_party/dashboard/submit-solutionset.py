#!/usr/bin/python3
# Copyright 2021 Team Special Weekend
# Copyright 2021 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


import argparse
import json
import os
import sys
import urllib.request

def yes_no_input():
    while True:
        choice = input("Submit solutions? 'yes' or 'no' [y/N]: ").lower()
        if choice in ['y', 'ye', 'yes']:
            return True
        elif choice in ['n', 'no']:
            return False

parser = argparse.ArgumentParser()
parser.add_argument('solution_set',
    metavar='SOLUTION_ID',
    type=str,
    help='solution set like 1,10,42')

args = vars(parser.parse_args())
solution_set = [s.strip() for s in args["solution_set"].split(",")]

api_token = os.environ['API_TOKEN']

# Download all solutions.
solutions = []
for sid in solution_set:
    url = "http://spweek.badalloc.com/api/solutions/{}".format(sid)
    with urllib.request.urlopen(url) as res:
        solutions.append(res.read().strip())

# Sanity check
problem_set = set()
solution_set = set()
for sol in solutions:
    s = json.loads(sol)
    print(s)
    problem_id = s['problem_id']
    solution_id = s['solution_id']
    if problem_id in problem_set:
        print("solution set has two solutions for problem {}, solution_id={}".format(
            problem_id, solution_id))
        sys.exit()
    if solution_id in solution_set:
        print("solution set has two same solution {}".format(solution_id))
        sys.exit()
    problem_set.add(problem_id)
    solution_set.add(solution_id)

print("Found {} solutions for {} problems".format(len(solution_set), len(problem_set)))

if not yes_no_input():
    sys.exit()

# Submit all
for sol in solutions:
    s = json.loads(sol)
    problem_id = s['problem_id']
    data = s['data']
    submission_url = "https://poses.live/api/problems/{}/solutions".format(problem_id)

    request = urllib.request.Request(submission_url)
    request.add_header('Content-Type', 'application/json')
    request.add_header('Authorization', 'Bearer {}'.format(api_token))

    with urllib.request.urlopen(request, json.dumps(data).encode("utf-8")) as response:
        print(response.read().decode("utf-8"))
