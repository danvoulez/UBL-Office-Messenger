#!/usr/bin/env python3
import json, sys
from hashlib import blake2b

CONTAINERS = [
  "C.Messenger",
  "C.Jobs",
  "C.Office",
  "C.Policy",
  "C.Runner",
]

def h32(s: str) -> str:
  return blake2b(s.encode(), digest_size=16).hexdigest()

out = { name: h32(name) for name in CONTAINERS }
json.dump(out, sys.stdout, indent=2)
