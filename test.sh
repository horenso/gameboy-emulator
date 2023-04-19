trap : INT
cd gameboy-doctor
cargo run ../cartridges/01-special.gb | ./gameboy-doctor - cpu_instrs 1
