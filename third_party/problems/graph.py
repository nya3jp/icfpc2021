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

import json

m = dict()
for i in range(1, 107):
    with open("{}.problem".format(i), 'r') as f:
        obj = json.load(f)
        if 'bonuses' not in obj:
            continue
        for bonus in obj['bonuses']:
            dest = bonus['problem']
            kind = bonus['bonus']
            if dest not in m:
                m[dest] = []
            m[dest].append((i, kind))

print('digraph g1 {')
kinds = ['GLOBALIST', 'BREAK_A_LEG', 'WALLHACK']
for dest, vs in m.items():
    for v in vs:
        src, kind = v
        if kind not in kinds:
            continue
        print('  P{} -> P{}'.format(src, dest))
print('}')
