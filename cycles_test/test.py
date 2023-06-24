#! /usr/bin/env python3

import json
from typing import Mapping, Set, Tuple


def pre(code: str) -> str:
    return f"pre{code}"


def getExpectationsAndMnemonics(path: str) -> Tuple[Mapping[str, set[int]], Mapping[str, str]]:
    with open(path, "r") as f:
        data = json.load(f)
        unprefixed = data["unprefixed"]
        prefixed = data["cbprefixed"]

    expectations: Mapping[str, Set[int]] = {
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
    return (expectations, mnemonics)


def getRecording(path: str) -> Mapping[str, Set[int]]:
    recording: Mapping[str, Set[int]] = dict()
    with open(path, "r") as f:
        for line in f.readlines():
            if line.startswith("pre"):
                _, code, cycles = line.split(" ")
                code = pre(code)
            else:
                code, cycles = line.split(" ")
            if code not in recording:
                recording[code] = set()
            recording[code].add(int(cycles))
    return recording


def check(expectations: Mapping[str, set[int]], recording: Mapping[str, Set[int]], mnemonics: Mapping[str, str]):
    failed = 0
    for opcode in recording:
        for cycle in recording[opcode]:
            if cycle not in expectations[opcode]:
                failed += 1
                print(
                    f"Opcode {opcode} "
                    f"{mnemonics[opcode]} took "
                    f"{cycle} (others {recording[opcode]}) "
                    f"but expects {expectations[opcode]}"
                )

    if failed > 0:
        print(f"Test failed: {failed}/{len(recording)} ")
        exit(1)


if __name__ == '__main__':
    expectations, mnemonics = getExpectationsAndMnemonics("./opcodes.json")
    recording = getRecording("mine.txt")
    check(expectations, recording, mnemonics)
