#!/bin/bash

cargo build --release

mkdir ./build_rust

cp target/release/tales_of_kathay ./build_rust/tales_of_kathay

find assets/ -name '*.png' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.mp3' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.ttf' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.toml' -exec cp --parents \{\} ./build_rust/ \;

cd ./build_rust/
zip -r release.zip ./
cd -
cp ./build_rust/release.zip ./
rm -rf ./build_rust/