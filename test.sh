# Working: 1, 3, 4, 5, 6, 7, 8

cd gameboy-doctor
cargo run ../cartridges/01-special.gb | ./gameboy-doctor - cpu_instrs 1
