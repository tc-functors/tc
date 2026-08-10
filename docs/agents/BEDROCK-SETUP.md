# Bedrock reviewer setup (Anthropic Claude on Amazon Bedrock)

End-to-end runbook to power the second reviewer (`scripts/ai_review.py`,
`.github/workflows/ai-review.yml`) with **Claude on Amazon Bedrock**. Inference stays
inside AWS — the diff is never sent to Anthropic's own API — and GitHub authenticates
to AWS with **OIDC** (no long-lived keys). No direct-Anthropic account needed.

Placeholders: `ACCOUNT_ID`, `REGION` (e.g. `us-west-2`), `ORG/REPO` (e.g.
`tc-functors/tc`). Do the AWS steps with an admin/role that can manage IAM + Bedrock.

---

## Part A — AWS

### A1. Enable Claude model access in Bedrock
Console → **Bedrock → Model access** (in `REGION`) → enable the Anthropic Claude
model(s) you want (some require a one-time access request/EULA). Do this in the region
you'll call from.

### A2. Pick the model id (usually an inference profile)
Claude on Bedrock is normally invoked through a **cross-region inference profile**, not
the bare foundation-model id. List what's available:
```sh
aws bedrock list-inference-profiles --region REGION \
  --query "inferenceProfileSummaries[?contains(inferenceProfileId,'anthropic')].inferenceProfileId"
# e.g. us.anthropic.claude-<name>-v1:0   (region-prefixed: us./eu./apac.)
```
Use that id as `AI_REVIEW_MODEL`. (A plain `anthropic.claude-...` id also works in
regions where on-demand is supported, but the profile is the safe default.)

### A3. Create the GitHub OIDC identity provider (once per account)
Skip if `token.actions.githubusercontent.com` is already an IAM OIDC provider.
```sh
aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --client-id-list sts.amazonaws.com
```

### A4. Create the reviewer IAM role
Trust policy (`trust.json`) — lets **only** this repo's Actions assume the role via
OIDC. Scope `sub` tighter (`:pull_request`) once confirmed working; `:*` is the
broad-but-repo-locked start:
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": { "Federated": "arn:aws:iam::ACCOUNT_ID:oidc-provider/token.actions.githubusercontent.com" },
    "Action": "sts:AssumeRoleWithWebIdentity",
    "Condition": {
      "StringEquals": { "token.actions.githubusercontent.com:aud": "sts.amazonaws.com" },
      "StringLike": { "token.actions.githubusercontent.com:sub": "repo:ORG/REPO:*" }
    }
  }]
}
```
Permissions policy (`perms.json`) — least-privilege invoke on the profile **and** the
foundation models it routes to (cross-region profiles span regions, hence the `*`):
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["bedrock:InvokeModel", "bedrock:InvokeModelWithResponseStream"],
    "Resource": [
      "arn:aws:bedrock:REGION:ACCOUNT_ID:inference-profile/us.anthropic.claude-*",
      "arn:aws:bedrock:*::foundation-model/anthropic.claude-*"
    ]
  }]
}
```
Create it:
```sh
aws iam create-role --role-name tc-ci-bedrock-review \
  --assume-role-policy-document file://trust.json
aws iam put-role-policy --role-name tc-ci-bedrock-review \
  --policy-name bedrock-invoke --policy-document file://perms.json
# Role ARN -> AI_REVIEW_AWS_ROLE below:
aws iam get-role --role-name tc-ci-bedrock-review --query Role.Arn --output text
```

---

## Part B — GitHub

Repo → **Settings → Secrets and variables → Actions → Variables** (these are
non-secret; the role ARN is not sensitive, and OIDC means **no API key/secret**):

| Variable | Value |
|---|---|
| `AI_REVIEW_BACKEND` | `bedrock` |
| `AI_REVIEW_MODEL` | the inference-profile id from A2 |
| `AI_REVIEW_AWS_REGION` | `REGION` |
| `AI_REVIEW_AWS_ROLE` | the role ARN from A4 |

That's it — `.github/workflows/ai-review.yml` already declares `id-token: write` and
runs `aws-actions/configure-aws-credentials` (assuming that role) only when
`AI_REVIEW_BACKEND == 'bedrock'`. Optional: require the **AI Review (second, pinned)**
check in branch protection, and/or set `AI_REVIEW_ENFORCE=1` in the workflow to make a
`VERDICT: BLOCK` fail the check (default is advisory).

---

## Part C — Validate

**Locally** (fastest sanity check, using your own AWS creds for `REGION`):
```sh
export AI_REVIEW_BACKEND=bedrock AI_REVIEW_AWS_REGION=REGION \
       AI_REVIEW_MODEL="us.anthropic.claude-<name>-v1:0"
python3 eval/run_eval.py --live        # scores the seed cases through Bedrock
```
**In CI:** open a small in-repo PR and watch the *AI Review (second, pinned)* job —
it should assume the role, call Bedrock, and post a review comment. Use
`bugbot run verbose=true` only for Bugbot; this reviewer logs to the Actions job.

---

## Troubleshooting
- **AccessDeniedException on invoke** → the IAM policy must cover the foundation-model
  ARNs the *profile* routes to (the `foundation-model/anthropic.claude-*` line), not
  just the profile ARN. For cross-region profiles keep the region wildcard.
- **ValidationException: model not enabled / not found** → enable it in Bedrock Model
  access (A1) in that region, and confirm the id is the *inference-profile* id.
- **OIDC "Not authorized to perform sts:AssumeRoleWithWebIdentity"** → the trust
  policy `sub` doesn't match. Fork PRs are excluded by design (the workflow gates to
  in-repo PRs); for in-repo PRs the `sub` is `repo:ORG/REPO:pull_request`.
- **Region mismatch** → `AI_REVIEW_AWS_REGION` must be a region where the model is
  enabled; the profile prefix (`us.`/`eu.`/`apac.`) must match that region family.
- **Nothing posts / job is a no-op** → the script no-ops (exit 0) if the backend isn't
  fully configured; check the four repo variables are set.

## Cost / privacy notes
- Billed as normal Bedrock model invocation on your account (per-token). Diffs are
  capped at `AI_REVIEW_MAX_DIFF` (default 60k chars) per review.
- Bedrock does not share prompts/completions with the model provider and does not use
  them for training; invocation logging is opt-in and stays in your account. Confirm
  against your AWS agreement / compliance policy.
