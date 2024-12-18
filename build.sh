#!/bin/bash

cargo build --release

mkdir /tmp/build_rust

cp target/release/package /tmp/build_rust/emergent_rpg
cp --parents assets/**/*.png /tmp/build_rust/
cp --parents assets/*.ttf /tmp/build_rust/

cd /tmp/build_rust/
zip -r release.zip ./
cd -
cp /tmp/build_rust/release.zip ./
rm -rf /tmp/build_rust/