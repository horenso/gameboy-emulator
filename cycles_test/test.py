#! /usr/bin/env python3

import json
from typing import Mapping, Set


def pre(code: str) -> str:
    return f"pre{code}"


with open("./opcodes.json") as f:
    data = json.load(f)
    unprefixed = data["unprefixed"]
    prefixed = data["cbprefixed"]

expectation: Mapping[str, Set[int]] = {
    **{
        k: set(v["cycles"]) for k, v in unprefixed.items()
    },
    **{
        pre(k): set(v["cycles"]) for k, v in prefixed.items()
    }
}
mnemonics: Mapping[str, str] = {
    **{
        k: v["mnemonic"] for k, v in unprefixed.items()
    },
    **{
        pre(k): v["mnemonic"] for k, v in prefixed.items()
    }
}

recording: Mapping[str, Set[int]] = dict()
with open("mine.txt", "r") as f:
    for line in f.readlines():
        if line.startswith("pre"):
            _, code, cycle = line.split(" ")
            code = pre(code)
        else:
            code, cycles = line.split(" ")
        if code not in recording:
            recording[code] = set()
        recording[code].add(int(cycles))

failed = 0
for opcode in recording:
    for cycle in recording[opcode]:
        if cycle not in expectation[opcode]:
            failed += 1
            print(
                f"Opcode {opcode} "
                f"{mnemonics[opcode]} took "
                f"{cycle} but expects "
                f"{expectation[opcode]}"
            )

if failed > 0:
    print(f"Test failed: {failed}/{len(recording)} ")
    exit(1)
