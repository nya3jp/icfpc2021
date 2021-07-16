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


cd "$(dirname "$0")"

if ! git diff-index --quiet HEAD --; then
    echo "Git checkout is dirty; refusing to make a package"
    exit 1
fi

name="chun-tachyon-solver.$(git rev-parse --short HEAD).tar.gz"

set -ex

cargo build --release

rm -rf .build
mkdir -p .build

cp -rH target/release/chun-tachyon-solver problems .build/

cd .build
tar czf "../$name" .

cd ..
rm -rf .build

set +x

echo "Package saved at $name"
echo "Use the following command to stage it:"
echo "gsutil cp $name gs://special-weekend-2021-flex/packages/"
