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

FROM docker.io/archlinux:latest as base-env
RUN pacman-key --init
RUN pacman --noconfirm -Sy extra/python3

FROM docker.io/node:latest as fe-build-env
WORKDIR /usr/src/app
COPY . .
RUN npm install
RUN npm run build

FROM base-env
COPY --from=fe-build-env /usr/src/app/build /build
ENTRYPOINT ["python3", "-m", "http.server", "8080", "-d", "/build"]
