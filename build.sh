#!/bin/bash

mkdir ./build_rust

find assets/ -name '*.png' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.mp3' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.wav' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.ttf' -exec cp --parents \{\} ./build_rust/ \;
find assets/ -name '*.toml' -exec cp --parents \{\} ./build_rust/ \;

# Linux build
cargo build --release
cp target/release/tales_of_kathay ./build_rust/tales_of_kathay

cd ./build_rust/
zip -r release-linux.zip ./
mv ./release-linux.zip ../
rm ./tales-of-kathay
cd -

# Windows build
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/tales_of_kathay.exe ./build_rust/tales_of_kathay.exe

cd ./build_rust/
zip -r release-windows.zip ./
mv ./release-windows.zip ../
rm ./tales-of-kathay.exe
cd -

rm -rf ./build_rust/