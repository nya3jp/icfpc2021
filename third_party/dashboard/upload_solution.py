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
import os
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('problem_id', metavar='PROBLEM_ID', type=int, help='problem ID')
parser.add_argument('solution', metavar='FILE', type=str, help='solution JSON file')
parser.add_argument('--tag', dest='tags', metavar='TAG', action='append', default=[], help='tag')
parser.add_argument('--url', default='http://spweek.badalloc.com/api/solutions', help='API URL')

args = parser.parse_args()
subprocess.run([
    'curl',
    '-F',
    'problem_id=' + str(args.problem_id),
    '-F',
    'tags=' + ','.join(args.tags),
    '-F',
    'solution=@' + args.solution,
    args.url,
])
