#!/bin/bash

if [ "$1" == "random" ]; then
    echo "Testing with random instructions"
    dd if=/dev/urandom of=random.gb bs=32K count=1
    rom=random.gb
else
    padded=$(printf "%02d" ${1})
    rom=./cartridges/${padded}.gb
fi

cargo run --release ${rom} 2> cycles_test/raw.txt

cd cycles_test
grep -E '^0x|^pre' raw.txt > mine.txt
# rm raw.txt
./test.py
# rm mine.txt
