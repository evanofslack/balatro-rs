"""Per-episode strategy logging and aggregate reporting.

Drives an env with a given agent's `run_episode`, and turns the resulting
`action_history` (now fully readable — see Stage 2's `Card`/`Jokers`
getters) into a per-episode record and an aggregate "what does this agent's
strategy look like" summary: ante reached, win rate, most-bought jokers,
action-kind mix, discard rate.
"""

import json
import math
import statistics
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
    # Resolved rank (HandRank.id(), e.g. "Flush"/"TwoPair") of every hand
    # actually played, in order — descriptive/logging only, mirrors
    # action_counts' shape. See core's Game.played_hands.
    hand_types: Counter = field(default_factory=Counter)


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
    hand_types = Counter(hr.id() for hr in state.played_hands)
    return EpisodeLog(
        seed=seed,
        won=env._game.is_win,
        ante_reached=state.round,
        steps=len(state.action_history),
        final_score=state.score,
        action_counts=action_counts,
        jokers_bought=jokers_bought,
        hand_types=hand_types,
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
    all_hand_types: Counter = Counter()
    discards = plays = 0
    for log in logs:
        all_jokers.update(log.jokers_bought)
        all_actions.update(log.action_counts)
        all_hand_types.update(log.hand_types)
        discards += log.action_counts.get("DiscardHand", 0) + log.action_counts.get(
            "Discard", 0
        )
        plays += log.action_counts.get("PlayHand", 0) + log.action_counts.get("Play", 0)

    scores = [log.final_score for log in logs]
    antes = [log.ante_reached for log in logs]
    # Averages alone can't tell "9% win rate" apart from "9 solid wins" vs.
    # "9 near-misses" — these min/median/max/stdev pairs, and the single
    # best_episode, give that visibility. ante_reached is the actual
    # win-progress metric (blinds/rounds cleared, per Game.round), so it's
    # the primary sort key for best_episode; final_score is the tiebreak.
    best_episode = max(
        logs, key=lambda log: (log.ante_reached, log.final_score), default=None
    )

    return {
        "episodes": n,
        "win_rate": rate,
        "win_rate_95ci": (lo, hi),
        "avg_ante_reached": sum(antes) / n if n else 0.0,
        "avg_steps": sum(log.steps for log in logs) / n if n else 0.0,
        "avg_final_score": sum(scores) / n if n else 0.0,
        "final_score_min": min(scores) if scores else 0,
        "final_score_median": statistics.median(scores) if scores else 0,
        "final_score_max": max(scores) if scores else 0,
        "final_score_stdev": statistics.stdev(scores) if len(scores) > 1 else 0.0,
        "ante_reached_min": min(antes) if antes else 0,
        "ante_reached_median": statistics.median(antes) if antes else 0,
        "ante_reached_max": max(antes) if antes else 0,
        "ante_reached_stdev": statistics.stdev(antes) if len(antes) > 1 else 0.0,
        "best_episode": (
            {
                "seed": best_episode.seed,
                "ante_reached": best_episode.ante_reached,
                "final_score": best_episode.final_score,
            }
            if best_episode is not None
            else None
        ),
        "discard_rate": discards / (discards + plays) if (discards + plays) else 0.0,
        "top_jokers_bought": all_jokers.most_common(10),
        "action_kind_histogram": dict(all_actions),
        "hand_type_histogram": dict(all_hand_types),
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
                    if k not in ("action_counts", "jokers_bought", "hand_types")
                },
                "action_counts": dict(log.action_counts),
                "jokers_bought": dict(log.jokers_bought),
                "hand_types": dict(log.hand_types),
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
    print(
        f"avg ante reached: {summary['avg_ante_reached']:.2f}  "
        f"(min {summary['ante_reached_min']} / median {summary['ante_reached_median']:g} / "
        f"max {summary['ante_reached_max']}, stdev {summary['ante_reached_stdev']:.2f})"
    )
    print(f"avg steps:       {summary['avg_steps']:.1f}")
    print(
        f"avg final score: {summary['avg_final_score']:.1f}  "
        f"(min {summary['final_score_min']:g} / median {summary['final_score_median']:g} / "
        f"max {summary['final_score_max']:g}, stdev {summary['final_score_stdev']:.1f})"
    )
    if summary["best_episode"]:
        b = summary["best_episode"]
        print(
            f"best episode:    seed={b['seed']}, ante_reached={b['ante_reached']}, "
            f"final_score={b['final_score']}"
        )
    print(f"discard rate:    {summary['discard_rate']:.1%}")
    print("top jokers bought:", summary["top_jokers_bought"])
    print("action kind mix:", summary["action_kind_histogram"])
    print("hand type mix:", summary["hand_type_histogram"])
    return summary
