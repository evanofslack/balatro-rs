"""Fixed held-out seed set for apples-to-apples agent comparison.

Every agent (random baseline, MCTS, future learned policies) should be
evaluated against exactly this set, not the training distribution, so
results are comparable across runs and across agents.
"""

EVAL_SEEDS = list(range(1_000, 1_100))
