#!/usr/bin/env python3
import json

def test(n, name):
    with open(f"{name}.json", "w") as file:
        json.dump({"value": n}, file)

for i in range(70):
    test({"a" * i: i}, "a" * i)
    test({"a" * i + "😃": i}, "a" * i + "😃")
