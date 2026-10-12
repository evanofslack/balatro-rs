"""Minimal line-protocol policy for balatro-agent-run (no model required).

Replace choose_action with checkpoint inference for a trained policy. All
diagnostics belong on stderr; stdout must contain one JSON reply per request.
"""

import json
import sys


def choose_action(state):
    mask = state["action_mask"]
    # Progress through blind, cash-out, and shop before ordinary hand actions.
    for index in (78, 72, 77, 70):
        if index < len(mask) and mask[index]:
            return index
    return next(index for index, legal in enumerate(mask) if legal)


for line in sys.stdin:
    try:
        print(json.dumps({"action": choose_action(json.loads(line))}), flush=True)
    except Exception as exc:
        print(json.dumps({"error": str(exc)}), flush=True)
        sys.exit(1)
