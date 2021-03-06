# Team Special Weekend ICFPC 2021

## Solvers

**tanakh-solver:**

his solver minimizes the value of dislike using the simulated annealing. As a
solution, we allow the ones that do not satisfy the length constraint in the
search, and add the ones that do not satisfy the length constraint as a penalty
to the value of dislike, which is used as the score. For the neighbors, in
addition to moving one point to a nearby coordinate, we used a parallel shifted
edge and a parallel shifted triangle. Calculating all the dislike values would
take about O(|hole|×|vertex|) time, which is a bottleneck for large problems.
Therefore, by considering only the cases where the modified vertex contributes
to the value of dislike, we accelerated the computation so that it takes only
O(|home|+|vertex|) on average. For bonuses, we made GLOBALIST and SUPERFLEX
available, which are easy to incorporate into our simulated annealing solver and
are relatively effective. The other two are not used.

**chun-bruteforce-server:**

**chun-oikomi:**

**chun-tachyon-solver:**

Solvers that exhaustively search solutions for problems with a small number of
vertices were also developed under following directories. These solvers try to
minimize the dislike score with exhaustive searches. These solvers were also
used to refine the results generated by simulated annealing and/or human,  by
finding solutions that are close to the existing one.

## Human-powered solving tools

We created a web application of a tool to lay out vertices by human mouse
operation. The method of having a human do the global optimization as a
preprocessing step and passing it to a solver such as simulated annealing as an
initial solution worked very well.

## Bonus usage strategies

We calculated scores with and without using bonuses for each problem. Then, we
sought combinations that satisfy the constraints of the bonuses and employed the
combination that gets the best total score.

