#!/usr/bin/env python3
import json
import base64

def test(n, name):
    value = base64.b64encode(n).decode("utf8")
    with open(f"{name}.json", "w") as file:
        json.dump({"value": value}, file)
    with open(f"{name}-raw.json", "w") as file:
        json.dump({"rawBinary": True, "value": value}, file)

test(b"", "empty")

for i in range(1, 9):
    test(b"a" * i, "a" * i)
    test(b"\xaa" * i, f"{i}pattern")
