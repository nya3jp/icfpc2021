#!/bin/bash
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


cd $(dirname $BASH_SOURCE)
cd ..

upload() {
  problem_id=$1
  solution=$2
  curl -F problem_id=$problem_id -F solution=@$solution http://localhost:8080/api/solutions
}

upload 27 chun-bruteforce-solver/results/27-2423-10061623.json
upload 28 chun-bruteforce-solver/results/28-3216-10061800.json
upload 29 chun-bruteforce-solver/results/29-3114-10062017.json
upload 30 chun-bruteforce-solver/results/30-1837-10134541.json
upload 30 chun-bruteforce-solver/results/30-2442-10065022.json
upload 36 chun-bruteforce-solver/results/36-1444-10065117.json
upload 40 chun-bruteforce-solver/results/40-3768-10134935.json
upload 1 tanakh-solver/results/1-1016-10033312.json
upload 1 tanakh-solver/results/1-1024-10031851.json
upload 1 tanakh-solver/results/1-1071-10032932.json
upload 1 tanakh-solver/results/1-1109-10031713.json
upload 1 tanakh-solver/results/1-1136-10040352.json
upload 1 tanakh-solver/results/1-1152-10032719.json
upload 1 tanakh-solver/results/1-1181-10031326.json
upload 1 tanakh-solver/results/1-1215-10031520.json
upload 1 tanakh-solver/results/1-1219-10032138.json
upload 1 tanakh-solver/results/1-1503-10032825.json
upload 1 tanakh-solver/results/1-3202-10022343.json
upload 1 tanakh-solver/results/1-695-10084039.json
upload 1 tanakh-solver/results/1-695-10085017.json
upload 1 tanakh-solver/results/1-696-10052604.json
upload 1 tanakh-solver/results/1-707-10050334.json
upload 1 tanakh-solver/results/1-710-10051030.json
upload 1 tanakh-solver/results/1-721-10051554.json
upload 1 tanakh-solver/results/1-745-10022524.json
upload 1 tanakh-solver/results/1-745-10084615.json
upload 1 tanakh-solver/results/1-750-10111003.json
upload 1 tanakh-solver/results/1-754-10090047.json
upload 1 tanakh-solver/results/1-794-10032535.json
upload 1 tanakh-solver/results/1-794-10084146.json
upload 1 tanakh-solver/results/1-812-10040609.json
upload 1 tanakh-solver/results/1-833-10084251.json
upload 1 tanakh-solver/results/1-834-10033449.json
upload 1 tanakh-solver/results/1-857-10083915.json
upload 1 tanakh-solver/results/1-876-10041006.json
upload 1 tanakh-solver/results/1-887-10034307.json
upload 1 tanakh-solver/results/1-887-10090205.json
upload 1 tanakh-solver/results/1-915-10033156.json
upload 1 tanakh-solver/results/1-918-10034238.json
upload 1 tanakh-solver/results/1-937-10032424.json
upload 1 tanakh-solver/results/1-971-10040459.json
upload 1 tanakh-solver/results/1-987-10033043.json
upload 1 tanakh-solver/results/1-992-10032257.json
upload 10 tanakh-solver/results/10-399-10063526.json
upload 10 tanakh-solver/results/10-415-10062306.json
upload 10 tanakh-solver/results/10-477-10041414.json
upload 10 tanakh-solver/results/10-52-10105825.json
upload 10 tanakh-solver/results/10-887-10025525.json
upload 11 tanakh-solver/results/11-0-10035044.json
upload 12 tanakh-solver/results/12-0-10035053.json
upload 21 tanakh-solver/results/21-505-10023117.json
upload 3 tanakh-solver/results/3-451-10022839.json
upload 3 tanakh-solver/results/3-813-10022727.json
upload 30 tanakh-solver/results/30-1942-10030132.json
upload 30 tanakh-solver/results/30-1943-10112047.json
upload 30 tanakh-solver/results/30-1944-10025919.json
upload 30 tanakh-solver/results/30-1963-10025704.json
upload 33 tanakh-solver/results/33-3612-10030111.json
upload 33 tanakh-solver/results/33-4038-10024513.json
upload 33 tanakh-solver/results/33-4099-10025645.json
upload 35 tanakh-solver/results/35-480-10080337.json
upload 37 tanakh-solver/results/37-1683-10040631.json
upload 37 tanakh-solver/results/37-2275-10040125.json
upload 4 tanakh-solver/results/4-1-10104609.json
upload 4 tanakh-solver/results/4-1-10105137.json
upload 4 tanakh-solver/results/4-23-10103913.json
upload 4 tanakh-solver/results/4-29-10092252.json
upload 4 tanakh-solver/results/4-6-10030442.json
upload 40 tanakh-solver/results/40-5282-10032509.json
upload 40 tanakh-solver/results/40-5282-10064011.json
upload 40 tanakh-solver/results/40-5495-10032727.json
upload 43 tanakh-solver/results/43-1259-10025244.json
upload 43 tanakh-solver/results/43-872-10025013.json
upload 44 tanakh-solver/results/44-10191-10024841.json
upload 44 tanakh-solver/results/44-11097-10034416.json
upload 44 tanakh-solver/results/44-8706-10034527.json
upload 45 tanakh-solver/results/45-6379-10065041.json
upload 48 tanakh-solver/results/48-4592-10042610.json
upload 49 tanakh-solver/results/49-0-10042030.json
upload 5 tanakh-solver/results/5-1019-10032837.json
upload 5 tanakh-solver/results/5-1132-10025408.json
upload 5 tanakh-solver/results/5-1523-10081413.json
upload 5 tanakh-solver/results/5-285-10080526.json
upload 5 tanakh-solver/results/5-427-10082450.json
upload 5 tanakh-solver/results/5-578-10033048.json
upload 50 tanakh-solver/results/50-5547-10032248.json
upload 50 tanakh-solver/results/50-5741-10031626.json
upload 50 tanakh-solver/results/50-6155-10032023.json
upload 50 tanakh-solver/results/50-6476-10031414.json
upload 50 tanakh-solver/results/50-8204-10031917.json
upload 53 tanakh-solver/results/53-1350-10081425.json
upload 53 tanakh-solver/results/53-1567-10084224.json
upload 53 tanakh-solver/results/53-1655-10082628.json
upload 53 tanakh-solver/results/53-748-10051133.json
upload 54 tanakh-solver/results/54-2701-10023405.json
upload 55 tanakh-solver/results/55-1669-10043556.json
upload 55 tanakh-solver/results/55-22-10045648.json
upload 55 tanakh-solver/results/55-278-10044818.json
upload 55 tanakh-solver/results/55-902-10044313.json
upload 56 tanakh-solver/results/56-4650-10024434.json
upload 57 tanakh-solver/results/57-5441-10041655.json
upload 57 tanakh-solver/results/57-6366-10023412.json
upload 58 tanakh-solver/results/58-2378-10034701.json
upload 58 tanakh-solver/results/58-4321-10023217.json
upload 59 tanakh-solver/results/59-1065-10065051.json
upload 59 tanakh-solver/results/59-123-10090306.json
upload 59 tanakh-solver/results/59-1551-10041115.json
upload 59 tanakh-solver/results/59-1574-10023228.json
upload 59 tanakh-solver/results/59-320-10085239.json
upload 59 tanakh-solver/results/59-959-10064011.json
upload 6 tanakh-solver/results/6-2663-10053658.json
upload 6 tanakh-solver/results/6-2723-10103356.json
upload 6 tanakh-solver/results/6-2832-10102412.json
upload 6 tanakh-solver/results/6-3694-10102838.json
upload 6 tanakh-solver/results/6-4001-10025101.json
upload 60 tanakh-solver/results/60-1049-10091924.json
upload 60 tanakh-solver/results/60-492-10093107.json
upload 60 tanakh-solver/results/60-734-10092045.json
upload 61 tanakh-solver/results/61-14378-10094353.json
upload 62 tanakh-solver/results/62-11047-10094626.json
upload 63 tanakh-solver/results/63-0-10103709.json
upload 63 tanakh-solver/results/63-444-10094738.json
upload 64 tanakh-solver/results/64-31574-10113937.json
upload 64 tanakh-solver/results/64-70695-10094849.json
upload 65 tanakh-solver/results/65-52865-10095108.json
upload 65 tanakh-solver/results/65-7209-10120501.json
upload 66 tanakh-solver/results/66-7762-10095219.json
upload 67 tanakh-solver/results/67-286-10095137.json
upload 68 tanakh-solver/results/68-186235-10093039.json
upload 68 tanakh-solver/results/68-50008-10110329.json
upload 68 tanakh-solver/results/68-55623-10105736.json
upload 69 tanakh-solver/results/69-1047-10095020.json
upload 70 tanakh-solver/results/70-725-10094906.json
upload 71 tanakh-solver/results/71-21573-10113210.json
upload 71 tanakh-solver/results/71-36137-10094753.json
upload 72 tanakh-solver/results/72-4-10094519.json
upload 73 tanakh-solver/results/73-15-10112156.json
upload 73 tanakh-solver/results/73-99-10094403.json
upload 74 tanakh-solver/results/74-68677-10094248.json
upload 74 tanakh-solver/results/74-9128-10114354.json
upload 75 tanakh-solver/results/75-111922-10100227.json
upload 76 tanakh-solver/results/76-1535-10115442.json
upload 76 tanakh-solver/results/76-6699-10093907.json
upload 77 tanakh-solver/results/77-0-10113243.json
upload 77 tanakh-solver/results/77-109-10093757.json
upload 77 tanakh-solver/results/77-34-10092126.json
upload 77 tanakh-solver/results/77-4-10101337.json
upload 78 tanakh-solver/results/78-1506-10112625.json
upload 78 tanakh-solver/results/78-9446-10093520.json
upload 8 tanakh-solver/results/8-1040-10062946.json
upload 9 tanakh-solver/results/9-1768-10111127.json
upload 9 tanakh-solver/results/9-2027-10033652.json
upload 9 tanakh-solver/results/9-2456-10024910.json
