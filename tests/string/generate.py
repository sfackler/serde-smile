#!/usr/bin/env python3

import json

def test(n, name=None):
    if name is None:
        name = n
    with open(f"{name}.json", "w") as file:
        json.dump({"value": n}, file)

test("", "empty")

for i in range(1,70):
    test("a" * i)
    test("a" * i + "😃")
