#!/usr/bin/python3
import argparse
import os
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('solution_id', metavar='SOLUTION_ID', type=int, help='solution ID')
parser.add_argument('--tag', dest='tags', metavar='TAG', action='append', default=[], help='tag')

args = parser.parse_args()
for tag in args.tags:
    subprocess.run([
        'curl',
        '-X',
        'POST',
        'https://spweek.badalloc.com/api/solutions/{}/tags?tag={}'.format(args.solution_id, tag),
    ])
