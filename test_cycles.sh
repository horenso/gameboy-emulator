padded=$(printf "%02d" ${1})

timeout 3s cargo run --release ./cartridges/${padded}.gb 2> cycles_test/raw.txt

cd cycles_test
grep '^0x' raw.txt > mine.txt
rm raw.txt
./test.py
rm mine.txt