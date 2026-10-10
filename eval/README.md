# Reviewer eval harness (Phase 4)

"Is the reviewer up to the task?" — answered with numbers. This harness runs labeled
cases through a reviewer and scores **precision / recall / accuracy** (treating a
`BLOCK` verdict as the positive class), so you can compare reviewers (Cursor Bugbot vs
the pinned `scripts/ai_review.py`), tune the charter, and catch regressions.

## Cases (`eval/cases/*.json`)
Each case is a small labeled change:
- **violation** — plants a convention breach that MUST be caught (`expect_verdict: BLOCK`),
  tagged with the charter `rule` it violates.
- **clean** — an idiomatic change that must **not** be flagged (`expect_verdict: PASS`);
  these guard against the reviewer nagging the deliberate idioms in
  `docs/agents/STYLE.md` §10 (the usual reason teams distrust AI review).

Seed set covers: `anyhow` dependency, `thiserror` custom error enum, unbounded
`join_all`, free-fn→method — and clean cases exercising `.clone()`, `x.len() > 0`,
`s!`/`empty()`, and the derive-quartet/`pub`/`HashMap` data-struct conventions. Add
more as new violation patterns show up in real PRs (ideally mine them from history).

## Running
```sh
python3 eval/run_eval.py                    # mock mode (no API key) — validates plumbing + cases
python3 eval/run_eval.py --min-accuracy 1.0 # fail if below threshold (used in CI, mock)
ANTHROPIC_API_KEY=… AI_REVIEW_MODEL=… python3 eval/run_eval.py --live   # score the real pinned model
```
- **Mock mode** uses each case's `mock_output`; it needs no key, runs in CI, and is a
  regression check on the case files + runner (a malformed case fails it).
- **Live mode** calls the same pinned model as the CI reviewer (via
  `scripts/ai_review.py`), so the score reflects a real reviewer. Run it locally /
  ad-hoc to measure and compare; it is intentionally not in the default CI (cost + it
  needs the secret).

## Interpreting
- **Recall** low → the reviewer misses real violations (dangerous — tighten the
  charter / raise effort / try a stronger model).
- **Precision** low → the reviewer flags clean idiomatic code (noise — usually a
  DO-NOT-FLAG gap in `.cursor/BUGBOT.md`).
- The point is to make "up to the task" a measured number you can show a skeptic, and
  to keep it from regressing as the charter and models change.

## Relationship to the other layers
Layer 1 constitution (`docs/agents/`), layer 2 deterministic gate
(`scripts/agent-check.sh`), layer 3 adversarial review (`.cursor/BUGBOT.md` +
`scripts/ai_review.py`), **layer 4 = this harness that measures layer 3.**
