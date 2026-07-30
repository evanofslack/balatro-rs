"""Seed set reserved for hyperparameter tuning — disjoint from EVAL_SEEDS
(eval_seeds.py's 1000-1099), so gym/tune.py's search never sees or overfits
to the held-out set used for final agent-vs-agent comparisons.
"""

TUNING_SEEDS = list(range(2_000, 2_200))
