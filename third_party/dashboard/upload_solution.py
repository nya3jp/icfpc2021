#!/usr/bin/python3
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
