#!/usr/bin/env python3
import json

def test(n, name):
    with open(f"{name}.json", "w") as file:
        json.dump({"sharedStrings": True, "value": n}, file)

test([{"1": "a", "2": "b"} for i in range(10)], "ab")
test([{"1": str(i)} for i in range(100)] * 2, "large")
test([{"1": "repeated", "2": str(i)} for i in range(1300)], "evict")
test([{"a": str(i), "b": str(i + 1)} for i in range(1300)], "valid_back_refs")
test([{"1": "a" * i, "2": "a" * (i + 1)} for i in range(100)], "long_strings")
test([{"1": "a" * 2, "2": "a" * 2}, {"1": "a" * 65, "2": "a" * 65}, {"1": "a" * 64, "2": "a" * 64}, {"1": "a", "2": "a"}], "share_limit")
