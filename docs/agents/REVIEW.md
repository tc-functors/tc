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
re-run validation from a trusted in-repo branch — or run the **fork-PR conformance
triage** below, which brings the pinned reviewer to a fork PR safely and on demand.

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

**Full step-by-step Bedrock + OIDC runbook (model access, IAM role, trust +
permission policies, validation, troubleshooting):** `docs/agents/BEDROCK-SETUP.md`.

Fork-safe by construction: gated to in-repo PRs (`head.repo == repo`) because fork
PRs can't read secrets or assume the role. **Not yet validated against a live
key/role — do a first live run on an in-repo PR after configuring.**

## Fork-PR conformance triage (maintainer-triggered)
Fork PRs are the one place the framework is otherwise blind: the two model reviewers
are gated to in-repo PRs (a fork can't read `ANTHROPIC_API_KEY` or assume the OIDC
role), so an outside contributor only ever sees the deterministic gate — not the
house-style judgment that decides whether a change reads like `tc`.
`.github/workflows/pr-conformance-triage.yml` closes that gap **on a maintainer's
explicit request**, reusing `scripts/ai_review.py` in `AI_REVIEW_MODE=conformance`
(feedback mode).

**How a maintainer triggers it**
- Comment **`/triage`** on the PR (must be an OWNER / MEMBER / COLLABORATOR), or
- run the **PR Conformance Triage** workflow via *Actions → Run workflow* with the PR
  number (`workflow_dispatch`).

**What it posts** — ONE "🔎 Conformance triage report" comment combining:
1. the **deterministic gate** conclusion (read from the `Agent Conformance` check that
   already ran on the fork PR — not recomputed here), and
2. the **pinned reviewer's** findings, each mapped to the exact rule
   (`STYLE.md §<n>` / `BUGBOT.md BLOCK #<n>`) and the concrete in-scope fix.

It is **always advisory** (never fails a check, even with `AI_REVIEW_ENFORCE=1`) and
is signed as the KiroCrew agent.

**Why it is fork-safe by construction** — the recurring danger is a
`pull_request_target`-style flow that checks out a fork's HEAD and *runs* it (build
scripts, tests, `build.rs`) while holding secrets. This workflow structurally cannot:
1. **Trusted definition.** It triggers on `issue_comment` / `workflow_dispatch`, which
   run the workflow file from the **default branch** — never the fork's copy — so a
   fork PR can't edit the workflow to exfiltrate anything.
2. **Maintainer gate.** It runs only for a `/triage` from a maintainer (or a
   `workflow_dispatch`, which needs write access). A fork author can't self-trigger the
   privileged (Bedrock + write-token) run.
3. **No untrusted code executes.** The job checks out the **trusted base repo** (its
   own `ai_review.py` + charter) and reads the contribution **only as a text diff**
   (`git fetch …/head` + `git diff`). The OIDC role and write token are never in scope
   while any contributor-controlled code runs, because none runs here.
4. **The gate stays where execution is safe.** Compiling/linting/testing contributor
   code is exactly what `agent-conformance.yml` already does on fork PRs
   (`permissions: contents: read`, no secrets — nothing to steal). The triage only
   *reads* that gate's conclusion.

So the split is: **execute untrusted code only in a no-secret sandbox (the gate);
touch secrets only over untrusted *data* (the diff), never untrusted *code* (the
triage).**

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
