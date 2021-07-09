#!/usr/bin/python3
import argparse
import os
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('problem_id', metavar='N', type=int, help='problem ID')
parser.add_argument('solution', metavar='FILE', type=str, help='solution JSON file')
parser.add_argument('--tags', metavar='TAG', nargs='*', type=str, help='tag')
parser.add_argument('--solution_set', default='', type=str, help='optional solution set')

args = parser.parse_args()
subprocess.run([
    'curl',
    '-F',
    'problem_id=' + str(args.problem_id),
    '-F',
    'tags=' + ','.join(args.tags),
    '-F',
    'solution_set=' + args.solution_set,
    '-F',
    'solution=@' + args.solution,
    'http://spweek.badalloc.com/api/solutions',
])
