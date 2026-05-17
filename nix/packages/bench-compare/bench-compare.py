"""Compare two criterion baselines and render a PNG chart.

Reads `estimates.json` files from `$CRITERION_HOME` (default: target/criterion)
for two named baselines and produces a figure with two subplots
"""
import argparse
import json
import os
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np


def find_estimates(root: Path, baseline: str):
    """Return dict {bench_id: estimates_dict} for every estimates.json under
    `<root>/<group>/<id>/<baseline>/estimates.json`."""
    out = {}
    for path in root.rglob(f"*/{baseline}/estimates.json"):
        bench_id = "/".join(path.relative_to(root).parts[:-2])
        with path.open() as f:
            out[bench_id] = json.load(f)
    return out


def pick_unit(ns_value: float) -> tuple[float, str]:
    for divisor, unit in [(1e9, "s"), (1e6, "ms"), (1e3, "µs")]:
        if ns_value >= divisor:
            return divisor, unit
    return 1.0, "ns"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("old_baseline", help="reference / old baseline name")
    parser.add_argument("new_baseline", help="comparison / new baseline name")
    parser.add_argument(
        "-o",
        "--output",
        default="bench-compare.png",
        help="output PNG path (default: %(default)s)",
    )
    parser.add_argument(
        "--criterion-home",
        default=os.environ.get("CRITERION_HOME", "target/criterion"),
        help="criterion root dir (default: $CRITERION_HOME or target/criterion)",
    )
    parser.add_argument(
        "--title",
        default=None,
        help="figure title (default: auto)",
    )
    args = parser.parse_args()

    root = Path(args.criterion_home)
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 1

    data_old = find_estimates(root, args.old_baseline)
    data_new = find_estimates(root, args.new_baseline)

    if not data_old:
        print(f"error: no estimates for baseline '{args.old_baseline}' under {root}", file=sys.stderr)
        return 1
    if not data_new:
        print(f"error: no estimates for baseline '{args.new_baseline}' under {root}", file=sys.stderr)
        return 1

    benches = sorted(set(data_old) & set(data_new))
    missing = sorted((set(data_old) | set(data_new)) - set(benches))
    if missing:
        print(f"warning: skipping benches missing in one baseline: {missing}", file=sys.stderr)
    if not benches:
        print("error: no overlapping benches between baselines", file=sys.stderr)
        return 1

    means_a = np.array([data_old[b]["mean"]["point_estimate"] for b in benches])
    means_b = np.array([data_new[b]["mean"]["point_estimate"] for b in benches])
    err_a = np.array([data_old[b]["mean"]["standard_error"] for b in benches])
    err_b = np.array([data_new[b]["mean"]["standard_error"] for b in benches])

    divisor, unit = pick_unit(float(max(means_a.max(), means_b.max())))
    pct_change = (means_b - means_a) / means_a * 100.0

    short_labels = [b.split("/")[-1] for b in benches]

    fig, (ax1, ax2) = plt.subplots(
        2, 1, figsize=(max(8, len(benches) * 0.7), 9), constrained_layout=True
    )

    x = np.arange(len(benches))
    width = 0.4
    ax1.bar(
        x - width / 2,
        means_a / divisor,
        width,
        yerr=err_a / divisor,
        label=args.old_baseline,
        color="#5b8def",
        capsize=3,
    )
    ax1.bar(
        x + width / 2,
        means_b / divisor,
        width,
        yerr=err_b / divisor,
        label=args.new_baseline,
        color="#f0a060",
        capsize=3,
    )
    ax1.set_ylabel(f"mean time ({unit})")
    ax1.set_title("Absolute mean execution time")
    ax1.set_xticks(x)
    ax1.set_xticklabels(short_labels, rotation=30, ha="right")
    ax1.legend()
    ax1.grid(axis="y", alpha=0.3)

    colors = ["#2e9e57" if v < 0 else "#d04444" for v in pct_change]
    ax2.bar(x, pct_change, color=colors)
    ax2.axhline(0, color="black", linewidth=0.8)
    ax2.set_ylabel("change (%)")
    ax2.set_title(f"Relative change: {args.new_baseline} vs {args.old_baseline} (negative = faster)")
    ax2.set_xticks(x)
    ax2.set_xticklabels(short_labels, rotation=30, ha="right")
    for i, v in enumerate(pct_change):
        ax2.text(
            i,
            v + (0.5 if v >= 0 else -0.5),
            f"{v:+.1f}%",
            ha="center",
            va="bottom" if v >= 0 else "top",
            fontsize=8,
        )
    ax2.grid(axis="y", alpha=0.3)

    title = args.title or f"{args.new_baseline} vs {args.old_baseline}"
    fig.suptitle(title, fontsize=14, fontweight="bold")

    out = Path(args.output)
    fig.savefig(out, dpi=150)
    print(f"wrote {out.resolve()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
