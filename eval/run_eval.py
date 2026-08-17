#!/usr/bin/env python3
"""Reviewer eval harness for `tc` (Phase 4).

Answers "is the reviewer up to the task?" with numbers instead of opinion. It runs a
set of labeled cases — planted convention VIOLATIONS (should BLOCK) and CLEAN
idiomatic changes (should PASS, i.e. must NOT flag the deliberate idioms) — through a
reviewer and scores precision / recall / accuracy (BLOCK = positive).

Two modes:
  --mock   (default) use each case's `mock_output`. Validates the harness plumbing and
           acts as a regression fixture set — no API key needed, runs anywhere/CI.
  --live   call the pinned model via scripts/ai_review.py (needs ANTHROPIC_API_KEY +
           AI_REVIEW_MODEL, same as the CI reviewer). This measures a real reviewer.

Usage:
  python3 eval/run_eval.py                 # mock scoring of eval/cases/*.json
  python3 eval/run_eval.py --live          # score the pinned model
  python3 eval/run_eval.py --min-accuracy 0.9   # exit 1 if accuracy below threshold

Case format (eval/cases/*.json):
  { "name", "kind": "violation"|"clean", "expect_verdict": "BLOCK"|"PASS",
    "rule": "<charter rule id, for violations>", "diff": "<unified diff>",
    "mock_output": "<canned reviewer text ending in VERDICT: BLOCK|PASS>" }
"""

import argparse
import glob
import importlib.util
import json
import os
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load_reviewer():
    path = os.path.join(REPO_ROOT, "scripts", "ai_review.py")
    spec = importlib.util.spec_from_file_location("ai_review", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def review_live(reviewer, diff):
    # Delegate to the shared backend dispatcher so --live scoring uses the exact same
    # path (anthropic | bedrock), charter, and verdict rules as the CI reviewer.
    text = reviewer.run_review(diff)
    if text is None:
        raise SystemExit(
            "--live: reviewer backend not configured. Set AI_REVIEW_MODEL and the "
            "backend's credentials (ANTHROPIC_API_KEY, or AI_REVIEW_BACKEND=bedrock + AWS creds)."
        )
    return text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--live", action="store_true", help="call the pinned model instead of mock outputs")
    ap.add_argument("--cases-dir", default=os.path.join(REPO_ROOT, "eval", "cases"))
    ap.add_argument("--min-accuracy", type=float, default=None)
    args = ap.parse_args()

    files = sorted(glob.glob(os.path.join(args.cases_dir, "*.json")))
    if not files:
        raise SystemExit(f"no cases in {args.cases_dir}")

    # Always load the reviewer module: mock and live both use its shared
    # parse_verdict() so the harness and the CI reviewer agree on verdicts. (Importing
    # the module runs no I/O; it only executes on __main__.)
    reviewer = load_reviewer()
    tp = fp = tn = fn = 0
    rows = []
    for f in files:
        with open(f, encoding="utf-8") as fh:
            case = json.load(fh)
        expected = case["expect_verdict"].upper()
        if args.live:
            text = review_live(reviewer, case.get("diff", ""))
        else:
            text = case.get("mock_output", "")
        predicted = reviewer.parse_verdict(text)
        ok = predicted == expected
        # BLOCK is the positive class
        if expected == "BLOCK" and predicted == "BLOCK":
            tp += 1
        elif expected == "PASS" and predicted == "BLOCK":
            fp += 1
        elif expected == "PASS" and predicted == "PASS":
            tn += 1
        else:
            fn += 1
        rows.append((case["name"], case.get("kind", ""), expected, predicted, "ok" if ok else "MISS"))

    total = tp + fp + tn + fn
    correct = tp + tn
    accuracy = correct / total if total else 0.0
    precision = tp / (tp + fp) if (tp + fp) else 1.0
    recall = tp / (tp + fn) if (tp + fn) else 1.0

    print(f"{'case':40} {'kind':10} {'expect':7} {'got':7} result")
    print("-" * 78)
    for name, kind, exp, got, res in rows:
        print(f"{name:40} {kind:10} {exp:7} {got:7} {res}")
    print("-" * 78)
    print(f"mode={'live' if args.live else 'mock'}  cases={total}  "
          f"accuracy={accuracy:.2f}  precision={precision:.2f}  recall={recall:.2f}  "
          f"(TP={tp} FP={fp} TN={tn} FN={fn})")

    if args.min_accuracy is not None and accuracy < args.min_accuracy:
        print(f"FAIL: accuracy {accuracy:.2f} < min {args.min_accuracy:.2f}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
