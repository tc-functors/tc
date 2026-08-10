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
- **Effort level (recommended: High):** in the Cursor dashboard → Bugbot Automations,
  set the review **effort** to **High** for this repo. Higher effort spends more
  reasoning per review (better recall on subtle style violations) at higher cost —
  worth it for a conformance gate. `Default` optimizes for speed and finds fewer bugs.

## Deterministic rules stay in the gate, not the model
Everything mechanically checkable — `no anyhow/thiserror`, `no unbounded join_all`,
fmt, line-scoped clippy, workspace tests — lives in `scripts/agent-check.sh` +
`agent-conformance.yml`, so reviewer *quality* never gates on those. The model
reviewers handle only the judgment a linter can't: "does this read like the original
author wrote it?" Keep it that way — when a rule becomes mechanically expressible,
move it from the charter into the gate.

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
## Second, pinned reviewer (GitHub Actions)
Bugbot's model is opaque and unpinnable. `.github/workflows/ai-review.yml` runs
`scripts/ai_review.py` — a small, stdlib-only reviewer that reads the **same charter**
(`AGENTS.md` + `docs/agents/STYLE.md` + `.cursor/BUGBOT.md`) so the two reviewers don't
diverge, and (unlike a diff-only bot) sends the style guide with the diff. It's
fail-safe (missing config or any API/CLI error → no-op, never breaks CI) and posts
findings as a PR comment; `VERDICT: BLOCK` can optionally enforce.

Two backends, chosen by the repo variable **`AI_REVIEW_BACKEND`**:

### `anthropic` (default — public Anthropic API)
Data egresses to Anthropic, so this is for public code / OSS repos.
1. Secret **`ANTHROPIC_API_KEY`**.
2. Variable **`AI_REVIEW_MODEL`** = your pinned Claude model id.

### `bedrock` (Amazon Bedrock — for private/company repos)
Inference runs **inside AWS** via the model-agnostic `aws bedrock-runtime converse`
CLI; per AWS's documented data handling, prompts/completions are **not** sent to the
model provider and are **not** used for training — so data stays within AWS. Setup:
1. Variables: `AI_REVIEW_BACKEND=bedrock`, `AI_REVIEW_MODEL` = a Bedrock model id /
   inference-profile ARN (Claude-on-Bedrock or Amazon Nova; Converse is model-agnostic
   so Llama/Mistral/OpenAI gpt-oss also work), and `AI_REVIEW_AWS_REGION`.
2. In AWS: enable model access for that model + region (Bedrock console → Model
   access; some models need an access request).
3. Auth via **GitHub OIDC → IAM role** (no static keys): register GitHub's OIDC
   provider in IAM; create a role scoped to this repo with `bedrock:InvokeModel` on
   the model/inference-profile ARN; set variable `AI_REVIEW_AWS_ROLE` to that role
   ARN. The workflow already declares `id-token: write` and runs
   `configure-aws-credentials` when the backend is `bedrock`.
4. For zero non-AWS hops even for the runner, use a self-hosted runner in your AWS
   account (GitHub-hosted runners are on Azure; either way the traffic is GitHub→AWS,
   which stays within "AWS or GitHub").

Optional (either backend): `AI_REVIEW_ENFORCE=1` makes a `BLOCK` verdict fail the
check (default advisory).

Fork-safe by construction: gated to in-repo PRs (`head.repo == repo`) because fork
PRs can't read secrets or assume the role. **Not yet validated against a live
key/role — do a first live run on an in-repo PR after configuring.**

## Compliance / data residency
Pick reviewers whose data handling fits the repo's rules:
- **Public / OSS repos** (like this one): any backend is fine.
- **Private repos with an egress boundary:** the **`bedrock`** backend keeps inference
  in AWS (data not sent to the model provider). **Cursor Bugbot** processes on Cursor's
  infrastructure — use it only where Cursor is an approved vendor for that repo. The
  **`anthropic`** backend sends the diff to Anthropic — use only where that egress is
  permitted. Confirm specifics against your own policy and cloud agreement.

## Measuring reviewer quality (eval harness)
"Is the reviewer up to the task?" is answered with numbers, not opinion — see the
`eval/` harness (Phase 4): it replays labeled cases (planted convention violations +
clean idiomatic changes) through a reviewer and scores precision/recall, so you can
compare Bugbot vs the pinned reviewer and catch regressions in the charter itself.

## Relationship to the other layers
1. Constitution — `AGENTS.md`, `docs/agents/STYLE.md|ARCHITECTURE.md|DSL.md`.
2. Deterministic gate — `scripts/agent-check.sh` + `agent-conformance.yml` (fmt,
   line-scoped clippy, convention greps, workspace tests).
3. **Adversarial review — this layer (`.cursor/BUGBOT.md`).**
4. (Planned) Eval harness — score agents against historical commits.
