#!/usr/bin/env python3
import json

def test(n):
    with open(f"{n}.json", "w") as file:
        json.dump({"value": n}, file)

test(True)
test(False)
