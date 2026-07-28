"""Average N same-version eval.py --out runs (different --agent-seed each)
into one summary-shaped JSON compare.py can read directly.

    python gym/aggregate.py results/mcts_100_v5_s*.json --out results/mcts_100_v5_avg.json
"""

import argparse
import json
from collections import Counter

NUMERIC_FIELDS = [
    "win_rate",
    "avg_ante_reached",
    "avg_steps",
    "avg_final_score",
    "discard_rate",
]


def aggregate(paths, out):
    runs = [json.load(open(p)) for p in paths]
    summaries = [r["summary"] for r in runs]
    n = len(summaries)

    avg_summary = {f: sum(s[f] for s in summaries) / n for f in NUMERIC_FIELDS}
    avg_summary["episodes"] = summaries[0]["episodes"]
    avg_summary["win_rate_95ci"] = (0.0, 0.0)  # not meaningful once averaged

    hist, jokers = Counter(), Counter()
    for s in summaries:
        hist.update(s["action_kind_histogram"])
        jokers.update(dict(s["top_jokers_bought"]))
    avg_summary["action_kind_histogram"] = dict(hist)
    avg_summary["top_jokers_bought"] = jokers.most_common(10)

    meta = {
        **runs[0]["meta"],
        "agent_seed": [r["meta"]["agent_seed"] for r in runs],
        "n_seeds": n,
    }
    with open(out, "w") as f:
        json.dump({"meta": meta, "summary": avg_summary}, f, indent=2)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("runs", nargs="+", help="results JSON files from eval.py --out")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    aggregate(args.runs, args.out)
