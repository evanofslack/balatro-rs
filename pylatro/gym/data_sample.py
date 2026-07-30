"""One training sample for the Stage 0 learned value model: a state (already
flattened via features.state_features), labeled with an outcome. Mirrors
logging_utils.py's EpisodeLog/save_results dataclass+JSON pattern.

v1 (rejected — see docs/mcts.md's Stage 0 diagnosis) labeled every sample
with its episode's real final outcome (final_score/won/ante_reached),
reflecting continued play by the full-strength agent — a mismatch with
model_value()'s actual call site (MctsAgent._rollout(), which evaluates
states under continued *weak rollout-policy* play). v2 labels with
mc_log_score instead (gym/rollout_value.py's Monte-Carlo estimate of
continued rollout-policy play from that exact state) — the corrected
target. final_score/won/ante_reached are kept (not removed) since they're
still meaningful metadata: for a real-decision-point sample, that state's
own real episode outcome; for a terminal-clone sample (a bonus row emitted
alongside mc_log_score's replicate playouts), its own real, well-defined
terminal outcome.
"""

import json
from dataclasses import asdict, dataclass, field
from typing import List, Optional


@dataclass
class DecisionSample:
    features: List[float] = field(default_factory=list)
    final_score: int = 0
    won: bool = False
    ante_reached: int = 0
    seed: int = 0
    # Position of this decision within its episode — not used by Stage 0's
    # plain Monte-Carlo-return training, but kept for later analysis (e.g.
    # weighting samples by recency within the episode).
    step_index: int = 0
    # v2's corrected training target (see module docstring). None for v1
    # data collected before this field existed.
    mc_log_score: Optional[float] = None


def save_samples(path: str, samples: List[DecisionSample]) -> None:
    with open(path, "w") as f:
        json.dump([asdict(s) for s in samples], f)


def load_samples(path: str) -> List[DecisionSample]:
    with open(path) as f:
        raw = json.load(f)
    return [DecisionSample(**r) for r in raw]
