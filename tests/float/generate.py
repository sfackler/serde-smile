#!/usr/bin/env python3
import json

def test(n):
    with open(f"{n}.json", "w") as file:
        json.dump({"value": n}, file)

test(0.0)
test(-0.0)
test(100.25)
test(-100.25)
