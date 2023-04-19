# Original Gameboy Emulator in Rust with SDL2.

To run it you need `libsdl2` and `Rust`.

```
cargo run path/to/cartridge
```

# Test
1. Put [test roms](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs/individual) into `cartridges/`
2. `./test.sh`

# References

https://gbdev.io/pandocs/

CPU instruction tables:
 - https://gbdev.io/gb-opcodes/optables/
 - https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html

CPU instruction descriptions:
 - https://github.com/rockytriton/LLD_gbemu/raw/main/docs/The%20Cycle-Accurate%20Game%20Boy%20Docs.pdf
 - http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
 - https://gekkio.fi/files/gb-docs/gbctr.pdf

Decoding CPU instructions structurally:
 - https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html

How does DDA work:
 - https://forums.nesdev.org/viewtopic.php?t=15944
