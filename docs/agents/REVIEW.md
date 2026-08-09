# Automated review layer (Phase 3)

This is the model-based **adversarial review** layer of the agent-conformance
framework. It catches what the deterministic gate (`scripts/agent-check.sh`,
`.github/workflows/agent-conformance.yml`) cannot: code that compiles and passes lint
but still doesn't read like `tc`'s Rust. The deterministic gate is the mechanical
floor; this layer is the taste/judgment gate.

## Cursor Bugbot (the reviewer already wired to this repo)
Bugbot reviews every PR and posts findings as PR comments (check name
`Cursor Bugbot`). It is configured for style-conformance review via a rule file.

- **Rule file:** `.cursor/BUGBOT.md` (repo root) is **always** included in every
  review. Nested `<dir>/.cursor/BUGBOT.md` files are included when the PR touches
  files under `<dir>` — use these for area-specific rules (e.g. a stricter charter for
  the disciplined layer under `lib/kit` or `lib/resolver`).
- **Important:** Cursor project rules (`.cursor/rules/*.mdc`) do **NOT** apply to
  Bugbot — they only steer the in-editor Cursor agent. So `.cursor/rules/tc-conventions.mdc`
  (added in Phase 1) guides authoring; `.cursor/BUGBOT.md` (this phase) guides review.
  Both are thin views of the same canonical source (`AGENTS.md` + `docs/agents/`).
- **The rule file has two halves on purpose:** a BLOCK list (catch convention-blind
  slop) and a DO-NOT-FLAG list (the deliberate house idioms in `docs/agents/STYLE.md`
  §10). The second half is what stops the bot from nagging about intentional style
  (`.clone()`, `x.len() > 0`, fail-fast `unwrap`), which is the usual reason teams
  distrust AI review.
- **See which rules a run used:** comment `bugbot run verbose=true` (or
  `cursor review verbose=true`) on a PR — Bugbot lists every rule it loaded.
- **Limits:** each rule is truncated at 30k chars, combined cap 100k. Keep
  `.cursor/BUGBOT.md` focused.
- **Blocking behavior:** by default Bugbot findings are non-blocking (`neutral`). To
  make unresolved findings fail the check, enable fail-on-unresolved-issues in the
  Cursor dashboard and require the `Cursor Bugbot` status in branch protection.

## Fork-PR caveat
Bugbot (and any secret-backed reviewer) does not run with full power on PRs from
forks. The **deterministic gate needs no secrets and does run on forks**, so it stays
the fork-safe baseline; model review is best-effort on forks and authoritative on
in-repo branches. A maintainer can always trigger a review manually (`bugbot run`) or
re-run validation from a trusted in-repo branch.

## Extending the review
- Add area-specific rules with nested `<dir>/.cursor/BUGBOT.md` files.
- Team/manual/learned rules can also be managed from the Cursor dashboard; inline
  `@cursor remember <fact>` on a PR teaches a durable rule.
- Optional future piece (not yet added): a second model reviewer as a GitHub Actions
  workflow (Anthropic/OpenAI), gated to in-repo branches because it needs API-key
  secrets. Keep it mirroring the same charter as `.cursor/BUGBOT.md` so the two
  reviewers don't diverge.

## Relationship to the other layers
1. Constitution — `AGENTS.md`, `docs/agents/STYLE.md|ARCHITECTURE.md|DSL.md`.
2. Deterministic gate — `scripts/agent-check.sh` + `agent-conformance.yml` (fmt,
   line-scoped clippy, convention greps, workspace tests).
3. **Adversarial review — this layer (`.cursor/BUGBOT.md`).**
4. (Planned) Eval harness — score agents against historical commits.
