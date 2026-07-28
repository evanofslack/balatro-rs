"""Floor baseline: sample uniformly among legal actions each step.

Uses `env.legal_actions()`/`env.step_action()` (the atomic-action interface),
not the fixed index mask, so it reflects the same action space MCTS uses.
"""

import random
from typing import Optional

AGENT_VERSION = "1"  # bump if action-selection logic ever changes


class RandomAgent:
    def __init__(self, rng: Optional[random.Random] = None):
        self._rng = rng or random.Random()

    def act(self, env):
        actions = list(env.legal_actions())
        return self._rng.choice(actions)

    def run_episode(self, env, seed: int, max_steps: int = 300):
        env.reset(seed=seed)
        terminated = truncated = False
        while not (terminated or truncated):
            action = self.act(env)
            _, _, terminated, truncated, _ = env.step_action(action)
        return env
