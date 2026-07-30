"""Compare two saved eval.py runs (e.g. before/after a change, or agent vs.
agent) side by side.

    python eval.py --agent random --out results/random.json
    python eval.py --agent mcts --sims 100 --out results/mcts_100.json
    python compare.py results/random.json results/mcts_100.json
"""

import argparse
import json


def load(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def skip_blind_rate(summary: dict) -> float:
    hist = summary.get("action_kind_histogram", {})
    skips = hist.get("SkipBlind", 0)
    selects = hist.get("SelectBlind", 0)
    total = skips + selects
    return skips / total if total else 0.0


def compare(paths):
    runs = [load(p) for p in paths]
    labels = [f"{r['meta']['agent']}" for r in runs]
    versions = [r["meta"].get("agent_version", "n/a") for r in runs]

    fields = [
        ("win_rate", "{:.1%}"),
        ("avg_ante_reached", "{:.2f}"),
        ("avg_steps", "{:.1f}"),
        ("avg_final_score", "{:.1f}"),
        ("discard_rate", "{:.1%}"),
        ("skip_blind_rate", "{:.1%}"),
    ]
    col_w = max(len(label) for label in labels) + 2
    print(f"{'metric':<18}" + "".join(f"{label:>{col_w}}" for label in labels))
    print(f"{'agent_version':<18}" + "".join(f"{v:>{col_w}}" for v in versions))
    for key, fmt in fields:
        row = []
        for r in runs:
            value = skip_blind_rate(r["summary"]) if key == "skip_blind_rate" else r["summary"][key]
            row.append(fmt.format(value))
        print(f"{key:<18}" + "".join(f"{v:>{col_w}}" for v in row))

    print()
    for path, r in zip(paths, runs):
        print(f"{path}: {r['summary']['episodes']} episodes, top jokers bought:",
              r["summary"]["top_jokers_bought"])
        print(f"  action_kind_histogram: {r['summary']['action_kind_histogram']}")
        print(f"  hand_type_histogram: {r['summary'].get('hand_type_histogram', {})}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("runs", nargs="+", help="results JSON files from eval.py --out")
    args = parser.parse_args()
    compare(args.runs)
