# Working: 1, 3, 4, 5, 6, 7, 8

padded=$(printf "%02d" ${1})

cd gameboy-doctor
cargo run --release ../cartridges/${padded}.gb | ./gameboy-doctor - cpu_instrs ${1}
