#!/usr/bin/env python3
import json

def test(n, name):
    with open(f"{name}.json", "w") as file:
        json.dump({"sharedStrings": True, "value": n}, file)

test([{"1": "a", "2": "b"} for i in range(10)], "ab")
test([{"1": str(i)} for i in range(100)] * 2, "large")
test([{"1": "repeated", "2": str(i)} for i in range(1300)], "evict")
