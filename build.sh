#!/bin/bash

cargo build --release

mkdir /tmp/build_rust

cp target/release/tales_of_kathay /tmp/build_rust/tales_of_kathay

find assets/ -name '*.png' -exec cp --parents \{\} /tmp/build_rust/ \;
find assets/ -name '*.mp3' -exec cp --parents \{\} /tmp/build_rust/ \;
find assets/ -name '*.ttf' -exec cp --parents \{\} /tmp/build_rust/ \;

cd /tmp/build_rust/
zip -r release.zip ./
cd -
cp /tmp/build_rust/release.zip ./
rm -rf /tmp/build_rust/