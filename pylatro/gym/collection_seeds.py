"""Seed set reserved for Stage 0 value-function data collection — disjoint
from both EVAL_SEEDS (1000-1099) and TUNING_SEEDS (2000-2199), so training
data for the learned value model never overlaps the held-out set used for
the final agent-vs-agent comparison in eval.py/compare.py. Overlap there
would let the model effectively memorize eval outcomes, making the A/B
meaningless (see docs/mcts.md's "Stage 0" plan).
"""

COLLECTION_SEEDS = list(range(3_000, 4_000))
