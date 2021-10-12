#!/usr/bin/env python3
import json

def test(n):
    with open(f"{n}.json", "w") as file:
        json.dump({"value": n}, file)

test(0)
test((1 << 64) - 1)
test(-((1 << 64) - 1))
test(1 << 120)
test(-(1 << 120))
