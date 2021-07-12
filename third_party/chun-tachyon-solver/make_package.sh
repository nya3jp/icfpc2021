#!/bin/bash

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
