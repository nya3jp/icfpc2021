import urllib.request
import json
import math

#PROBLEM_SIZE = 1
PROBLEM_SIZE = 132
#PROBLEM_SIZE = 20

# Base Data
problems = dict()
solutions = dict()

globalBestDislike = dict()
dependency = dict()
states = dict()
candidates = list()
diffData = dict()
decided = dict()
constraint = dict()
selected = list()
seenCandidates = dict()
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
    score = math.ceil(1000.0 * math.log2(getProblemSize(problemId)/6.0) * math.sqrt((globalBestDislike[problemId]+1.0)/(dislike+1.0)))
    return score

def updateDict(orig, target):
    if 'score' not in orig:
        return target
    if orig['score'] > target['score']:
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
            candidate['minimal_dislike'] = globalBestDislike[key]
            if len(submit[2]) == 0:
                if 'unused' not in diffData[key]:
                    diffData[key]['unused'] = candidate
                else:
                    diffData[key]['unused'] = updateDict(diffData[key]['unused'], candidate)
            if  'candidates' not in diffData[key]:
                diffData[key]['candidates'] = [candidate]
            else:
                diffData[key]['candidates'].append(candidate)
            seenCandidates[submit[0]] = False
            candidates.append(candidate)

def calcup(candidate):
    return candidate['score'] * 1.5 - diffData[candidate['problemId']]['unused']['score']
    
def calcdown(prob, bonus):
    ret = -9999999
    if prob not in diffData:
        print('diffData invalid')
        return 9999999
    print(len(diffData[prob]['candidates']))
    for item in diffData[prob]['candidates']:
        if seenCandidates[item['solutionId']]:
            print('Seen!')
            continue
        if len(item['catchedBonus']) == 0:
            print('dont catched Bonus')
            continue
        valid = True
        for b in item['catchedBonus']:
            tmp = False
            if b == bonus:
                tmp = True
            else:
                for c in constraint[prob]:
                    if b == c:
                        tmp = True
            valid = valid and tmp
        if valid:
            ret = max(ret, item['score'] - diffData[prob]['unused']['score'])
    if ret == -9999999:
        print('Invalid!')
        return 9999999
    else:
        print('Hit!')
        return ret

def decide(candidate):
    print("Candidate ", end="")
    print(candidate)
    seenCandidates[candidate['solutionId']] = True
    if candidate['problemId'] in decided:
        print("Rejected already decided")
        return

    # reject candidates that have invalid catch Bonus
    if (candidate['problemId'] in constraint) and (len(constraint[candidate['problemId']]) != 0):
        is_ret = True
        for i in  constraint[candidate['problemId']]:
            found = False
            for j in candidate['catchedBonus']:
                if j[0] == i[0]:
                    found = True
            is_ret = is_ret and found
        if not is_ret:
            seenCandidates[candidate['solutionId']] = True
            print("Rejected does not fit constraint")
            return
        else:
            print("Selected! fit all constraint")
            selected.append(candidate)
            decided[candidate['problemId']] = candidate
            if candidate['usedBonus'][0][0] not in constraint[candidate['problemId']]:
                    constraint[candidate['problemId']].append(candidate['usedBonus'][0][0])
        return
    else:
        constraint[candidate['problemId']] = []

    if len(candidate['usedBonus']) == 0:
        print("Selected! bonus is not enabled")
        selected.append(candidate)
        decided[candidate['problemId']] = candidate
        return

    bonus = candidate['usedBonus'][0][0]
    prob = candidate['usedBonus'][0][1]
    if prob not in constraint:
        constraint[prob] = []
    if prob in decided:
        found = False
        for catched in decided[prob]['catchedBonus']:
            if catched[0] == bonus:
                found = True
        if found:
            print("Selected! fit all constraint in previous selected item")
            selected.append(candidate)
            decided[candidate['problemId']] = candidate
        print("Rejected does not fit constraint in decided")
        return
    else:
        if calcup(candidate) > calcdown(prob, bonus):
            print("Selected! we need to get this item from cost")
            selected.append(candidate)
            decided[candidate['problemId']] = candidate
            if bonus not in constraint[prob]:
                constraint[prob].append(bonus)
        else:
            print("Rejected! we don't need to get this item from cost")

def solve():
    sortedCandidates = sorted(candidates, key=lambda x:(x['score'], -len(x['usedBonus']), len(x['catchedBonus'])), reverse=True)
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

def updateGlobalBestDislike(problemNumber, solution):
    for submit in solution:
        dislike = submit['dislike']
        globalBestDislike[problemNumber] = min(globalBestDislike[problemNumber], dislike)

def createDependency(problem):
    if 'bonuses' in problem['data'] and problem['data']['bonuses'] is not None:
        for bonus in problem['data']['bonuses']:
            if bonus['problem'] not in dependency:
                dependency[bonus['problem']] = []
            dependency[bonus['problem']].append((problem['problem_id'], bonus['bonus']))

def getURL(sortedSelected):
    url = 'https://spweek.badalloc.com/#/solutionsets/?id='
    for i in sortedSelected:
        url = url + str(i['solutionId']) + ','
    return url[0:-1] 


def main():
    # Get Problems and Solutions
    for i in range(PROBLEM_SIZE):
        problemNumber = i+1
        problems[problemNumber] = getProblemJson(problemNumber)
        solutions[problemNumber] = getSolutionsJson(problemNumber)
    for key, problem in problems.items():
        getGlobalBestDislike(problem)
        createDependency(problem)

    for problemNumber, solution in solutions.items():
        createStates(problemNumber, solution)
        updateGlobalBestDislike(problemNumber, solution)

    createDiffDataAndCandidate()
    solve()
    sortedSelected = sorted(selected, key=lambda x:x['problemId'], reverse=True)
    print(getURL(sortedSelected))
    totalScore = 0
    for i in selected:
        totalScore = totalScore + i['score']
    print(len(selected))
    print(totalScore)
    totalScore = 0
    count_total_candidate = 0
    seenCandidates = dict()
    for i in candidates:
        if len(i['usedBonus']) > 0:
            continue
        if i['problemId'] in seenCandidates:
            continue
        seenCandidates[i['problemId']] = True
        count_total_candidate = count_total_candidate + 1
        totalScore = totalScore + i['score']
    print(count_total_candidate)
    print(totalScore)

if __name__ == "__main__":
    main()
