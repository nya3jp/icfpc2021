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
