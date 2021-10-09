#!/usr/bin/env python3
import json

def test(n):
    with open(f"{n}.json", "w") as file:
        json.dump({"value": n}, file)

test(0)

v = 1
while v < 1 << 31:
    test(v)
    test(-v)
    even = v % 2 == 0
    v *= 2
    if even:
        v += 1
