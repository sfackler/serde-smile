#!/usr/bin/env python3
import json

def test(n, name):
    with open(f"{name}.json", "w") as file:
        json.dump({"sharedProperties": True, "value": n}, file)

test([{"a": 0, "b": 1} for i in range(10)], "ab")
test([{str(i): 0} for i in range(100)] * 2, "large")
test([{"repeated": 0, str(i): 1} for i in range(1300)], "evict")
