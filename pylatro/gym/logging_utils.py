"""Per-episode strategy logging and aggregate reporting.

Drives an env with a given agent's `run_episode`, and turns the resulting
`action_history` (now fully readable — see Stage 2's `Card`/`Jokers`
getters) into a per-episode record and an aggregate "what does this agent's
strategy look like" summary: ante reached, win rate, most-bought jokers,
action-kind mix, discard rate.
"""

import json
import math
import time
from collections import Counter
from dataclasses import asdict, dataclass, field
from typing import List


@dataclass
class EpisodeLog:
    seed: int
    won: bool
    ante_reached: int
    steps: int
    final_score: int
    action_counts: Counter = field(default_factory=Counter)
    jokers_bought: Counter = field(default_factory=Counter)


def _action_kind(action) -> str:
    # pyo3 complex-enum instances are typed "Action_<Variant>".
    name = type(action).__name__
    prefix = "Action_"
    return name[len(prefix) :] if name.startswith(prefix) else name


def record_episode(env, seed: int) -> EpisodeLog:
    state = env._game.state
    action_counts: Counter = Counter()
    jokers_bought: Counter = Counter()
    for action in state.action_history:
        kind = _action_kind(action)
        action_counts[kind] += 1
        if kind == "BuyJoker":
            jokers_bought[action._0.id()] += 1
    return EpisodeLog(
        seed=seed,
        won=env._game.is_win,
        ante_reached=state.round,
        steps=len(state.action_history),
        final_score=state.score,
        action_counts=action_counts,
        jokers_bought=jokers_bought,
    )


def wilson_interval(successes: int, n: int, z: float = 1.96):
    if n == 0:
        return (0.0, 0.0, 0.0)
    p = successes / n
    denom = 1 + z * z / n
    center = (p + z * z / (2 * n)) / denom
    half = (z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))) / denom
    return (p, max(0.0, center - half), min(1.0, center + half))


def summarize(logs: List[EpisodeLog]) -> dict:
    n = len(logs)
    wins = sum(1 for log in logs if log.won)
    rate, lo, hi = wilson_interval(wins, n)

    all_jokers: Counter = Counter()
    all_actions: Counter = Counter()
    discards = plays = 0
    for log in logs:
        all_jokers.update(log.jokers_bought)
        all_actions.update(log.action_counts)
        discards += log.action_counts.get("DiscardHand", 0) + log.action_counts.get(
            "Discard", 0
        )
        plays += log.action_counts.get("PlayHand", 0) + log.action_counts.get("Play", 0)

    return {
        "episodes": n,
        "win_rate": rate,
        "win_rate_95ci": (lo, hi),
        "avg_ante_reached": sum(log.ante_reached for log in logs) / n if n else 0.0,
        "avg_steps": sum(log.steps for log in logs) / n if n else 0.0,
        "avg_final_score": sum(log.final_score for log in logs) / n if n else 0.0,
        "discard_rate": discards / (discards + plays) if (discards + plays) else 0.0,
        "top_jokers_bought": all_jokers.most_common(10),
        "action_kind_histogram": dict(all_actions),
    }


def save_results(path: str, logs: List[EpisodeLog], summary: dict, meta: dict) -> None:
    payload = {
        "meta": {**meta, "timestamp": time.time()},
        "summary": summary,
        "episodes": [
            {
                **{
                    k: v
                    for k, v in asdict(log).items()
                    if k not in ("action_counts", "jokers_bought")
                },
                "action_counts": dict(log.action_counts),
                "jokers_bought": dict(log.jokers_bought),
            }
            for log in logs
        ],
    }
    with open(path, "w") as f:
        json.dump(payload, f, indent=2)


def print_report(logs: List[EpisodeLog], label: str = "agent") -> dict:
    summary = summarize(logs)
    lo, hi = summary["win_rate_95ci"]
    print(f"=== strategy report: {label} ({summary['episodes']} episodes) ===")
    print(f"win rate:        {summary['win_rate']:.1%}  (95% CI [{lo:.1%}, {hi:.1%}])")
    print(f"avg ante reached: {summary['avg_ante_reached']:.2f}")
    print(f"avg steps:       {summary['avg_steps']:.1f}")
    print(f"avg final score: {summary['avg_final_score']:.1f}")
    print(f"discard rate:    {summary['discard_rate']:.1%}")
    print("top jokers bought:", summary["top_jokers_bought"])
    print("action kind mix:", summary["action_kind_histogram"])
    return summary
