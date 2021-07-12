import urllib.request
import json
import math

#PROBLEM_SIZE = 1
PROBLEM_SIZE = 132
problems = dict()
solutions = dict()
globalBestDislike = dict()
dependency = dict()
states = dict()
candidates = list()
diffData = dict()
decided = dict()
selected = list()
kinds = ['GLOBALIST', 'BREAK_A_LEG', 'WALLHACK', 'SUPERFLEX']

def getSolutionsJson(i):
    response = urllib.request.urlopen('https://spweek.badalloc.com/api/problems/' + str(i) + '/solutions')
    html = response.read()
    print('Get Solutions for Problem ' + str(i))
    if response.getcode() != 200:
        print('Fail to get solutions for problem'+ str(i))
        return json.loads("[]")
    return json.loads(html.decode('utf-8'))

def createStates(problem, solution):
    if problem not in states:
        states[problem] = []
    for submit in solution:
        solutionId = submit['solution_id']
        dislike = submit['dislike']
        usedBonus = []
        if 'bonuses' in submit['data'] and submit['data']['bonuses'] is not None:
            for bonus in submit['data']['bonuses']:
                usedBonus.append((bonus['bonus'] , bonus['problem']))
        catchedBonus = []
        if 'bonuses' in problems[problem]['data']:
            for bonus in problems[problem]['data']['bonuses']:
                for v in submit['data']['vertices']:
                    if bonus['position'] == v:
                        catchedBonus.append((bonus['bonus'], bonus['problem']))
        states[problem].append((solutionId, dislike, usedBonus, catchedBonus))

def getProblemSize(problemId):
    return len(problems[problemId]['data']['hole']) * len(problems[problemId]['data']['figure']['vertices']) * len(problems[problemId]['data']['figure']['edges'])

def getScore(problemId, dislike):
    score = math.ceil(1000.0 * math.log2(getProblemSize(problemId)) * math.sqrt((globalBestDislike[problemId]+1.0)/(dislike+1.0)))
    return score

def updateDict(orig, target):
    if 'score' not in orig:
        return target
    if orig['socre'] > target('score'):
        return orig
    else:
        return target

def createDiffDataAndCandidate():
    for key, state in states.items():
        diffData[key] = dict()
        for submit in state:
            candidate = dict()
            candidate['score'] = getScore(key, submit[1])
            candidate['problemId'] = key
            candidate['solutionId'] = submit[0]
            candidate['catchedBonus'] = submit[3]
            candidate['usedBonus'] = submit[2]
            candidate['dislike'] = submit[1]
            for usedBonus in submit[2]:
                if usedBonus not in diffData[key]:
                    diffData[key][usedBonus[0]] = dict()
                for catchedBonus in submit[3]:
                    diffData[key][usedBonus[0]][catchedBonus[0]] = candidate
                if len(submit[3]) == 0:
                    diffData[key][usedBonus[0]]['unused'] = candidate
            if len(submit[2]) == 0:
                if 'unused' not in diffData[key]:
                    diffData[key]['unused'] = dict()
                for catchedBonus in submit[3]:
                    diffData[key]['unused'][catchedBonus[0]] = candidate
                if len(submit[3]) == 0:
                    diffData[key]['unused']['unused'] = candidate
            candidates.append(candidate)

def calcup(candidate, bonus):
    if 'unused' not in diffData[candidate['problemId']][bonus]:
        return candidate['score']
    else:
        return candidate['score'] - diffData[candidate['problemId']][bonus]['unused']['score']
    
def calcdown(diff, bonus):
    ret = 99999999
    for i, sc in diff.items():
        if i == 'unused':
            continue
        for j, catch in sc.items():
            if j == bonus:
                if 'unused' not in sc:
                    return 0
                else:
                    ret = diff[bonus]['unused'] - catch
    return ret

def decide(candidate):
    if candidate['problemId'] in decided:
        return
    if len(candidate['usedBonus']) == 0:
        print(candidate)
        selected.append(candidate)
        decided[candidate['problemId']] = candidate
        return
    bonus = candidate['usedBonus'][0][0]
    prob = candidate['usedBonus'][0][1]
    if prob in decided:
        found = False
        for catched in decided[prob]['catchedBonus']:
            if catched[0] == bonus:
                found = True
        if found:
            print(candidate)
            selected.append(candidate)
            decided[candidate['problemId']] = candidate
        return
    else:
        if prob not in diffData:
            return
        if calcup(candidate, bonus) > calcdown(diffData[prob], bonus):
            print(candidate)
            selected.append(candidate)
            decided[candidate['problemId']] = candidate

def solve():
    sortedCandidates = sorted(candidates, key=lambda x:x['score'], reverse=True)
    for candidate in sortedCandidates:
        decide(candidate)


def getProblemJson(i):
    response = urllib.request.urlopen('https://spweek.badalloc.com/api/problems/' + str(i))
    html = response.read()
    print('Get Problem for Problem ' + str(i))
    if response.getcode() != 200:
        print('Fail to get problems for problem'+ str(i))
        return json.loads("[]")
    return json.loads(html.decode('utf-8'))

def getGlobalBestDislike(problem):
    globalBestDislike[problem['problem_id']] = problem['minimal_dislike']

def createDependency(problem):
    if 'bonuses' in problem['data'] and problem['data']['bonuses'] is not None:
        for bonus in problem['data']['bonuses']:
            if bonus['problem'] not in dependency:
                dependency[bonus['problem']] = []
            dependency[bonus['problem']].append((problem, bonus['bonus']))

def main():
    totalscore = 0
    for i in range(PROBLEM_SIZE):
        problemNumber = i+1
        solutions[problemNumber] = getSolutionsJson(problemNumber)
        problems[problemNumber] = getProblemJson(problemNumber)
        for key, problem in problems.items():
            getGlobalBestDislike(problem)
            createDependency(problem)

    for problemNumber, solution in solutions.items():
        createStates(problemNumber, solution)
    createDiffDataAndCandidate()
    solve()
    print(len(selected))
    total = 0
    for i in selected:
        total = total + i['score']
    print(total)

if __name__ == "__main__":
    main()
