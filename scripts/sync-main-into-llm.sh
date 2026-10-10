#!/usr/bin/env bash
#
# sync-main-into-llm.sh — pull code + bug fixes from `main` into `main-llm`
# while preserving the agentic layer (configs, skills, docs) that `main`
# intentionally reverted.
#
# WHY THIS EXISTS
#   The agentic files were added on `main` (commit 9d518cbd) and then removed
#   again by a revert (commit 1f264e54 "Revert Agentic workflow test (#91)").
#   `main-llm` keeps them. Because that revert lives in `main`'s history as a
#   normal commit, a plain `git merge main` faithfully REPLAYS the deletion and
#   wipes the agentic files on `main-llm`. This script merges `main`, then
#   restores the agentic paths from a known-good tree, in one commit.
#
# WHAT IT DOES NOT NEED
#   An earlier plan called for `git merge -s ours <revert>` to "neutralize" the
#   revert. In practice the revert is already an ancestor of `main`'s tip, so a
#   normal merge of `main` already carries it in history and never re-proposes
#   the deletion on SUBSEQUENT merges. The only reason the files vanish is the
#   first time `main`'s tip (which contains the revert) is pulled in — and this
#   script restores them in that same step. No `-s ours` dance required.
#
# USAGE
#   scripts/sync-main-into-llm.sh                 # work on current main-llm
#   scripts/sync-main-into-llm.sh --dry-run       # show what would change, no commit
#
#   KNOWN_GOOD=<sha> scripts/sync-main-into-llm.sh   # restore the agentic layer
#       from a specific commit instead of the current main-llm tip. Use this if a
#       prior sync already stripped the agentic files from the tip; point it at a
#       commit that still has them. The script refuses to run if the chosen tree
#       lacks AGENTS.md, so it cannot silently merge the agentic layer away.
#
# After it finishes, review `git show HEAD` and `git log --oneline`, then push:
#   git push origin main-llm
# (or push a feature branch and open a PR if main-llm is protected).
#
# This script is deliberately NON-DESTRUCTIVE: it never runs `git reset --hard`,
# never force-pushes, and aborts cleanly if the working tree is dirty.

set -euo pipefail

SOURCE_BRANCH="${SOURCE_BRANCH:-main}"
TARGET_BRANCH="${TARGET_BRANCH:-main-llm}"
REMOTE="${REMOTE:-origin}"
DRY_RUN=0
[ "${1:-}" = "--dry-run" ] && DRY_RUN=1

# The agentic paths that `main`'s revert deletes and that we must preserve.
# Keep this list in sync with the revert's file set if the agentic layer grows.
AGENTIC_PATHS=(
  AGENTS.md
  CLAUDE.md
  .cursor
  .github/copilot-instructions.md
  .github/workflows/agent-conformance.yml
  .github/workflows/ai-review.yml
  .github/workflows/eval.yml
  .github/workflows/pr-conformance-triage.yml
  .kiro/settings/cli.json
  .kiro/settings/mcp.json
  .kiro/steering/tc-conventions.md
  docs/agents
  eval
  scripts/agent-check.sh
  scripts/ai_review.py
)

die() { echo "error: $*" >&2; exit 1; }

# --- preflight ---------------------------------------------------------------
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "not inside a git repo"

# Block only on uncommitted changes to TRACKED files (staged or unstaged) — those
# are what a merge/checkout can clobber or conflict with. Untracked files (stray
# locks, scratch html, this script before it is committed) are harmless to the
# sync, so we warn about them but do not refuse.
if ! git diff --quiet || ! git diff --cached --quiet; then
  die "working tree has uncommitted changes to tracked files; commit or stash before syncing"
fi

UNTRACKED="$(git ls-files --others --exclude-standard)"
if [ -n "$UNTRACKED" ]; then
  echo "note: ignoring these untracked files (they do not block the sync):"
  printf '%s\n' "$UNTRACKED" | while IFS= read -r f; do printf '  %s\n' "$f"; done
fi

git fetch "$REMOTE" --quiet
git switch "$TARGET_BRANCH"

# Choose the known-good agentic tree to restore from. By default this is the
# CURRENT target tip, captured BEFORE the merge touches anything. You may pin a
# specific commit with KNOWN_GOOD=<sha> for the case where the target branch has
# already (partially) merged main and its tip no longer contains the agentic
# layer — e.g. re-running after a failed sync.
KNOWN_GOOD="$(git rev-parse "${KNOWN_GOOD:-HEAD}")"

# Guard: refuse to proceed unless the chosen tree ACTUALLY contains the agentic
# layer. Without this, a KNOWN_GOOD pointing at an already-stripped tree would
# "restore" nothing and silently merge away the agentic files. We require a
# representative anchor file (AGENTS.md) to be present in that tree.
if ! git cat-file -e "$KNOWN_GOOD:AGENTS.md" 2>/dev/null; then
  die "known-good tree $KNOWN_GOOD does not contain the agentic layer (AGENTS.md missing).
       The target tip has probably already merged $SOURCE_BRANCH. Re-run with
       KNOWN_GOOD=<sha of a commit that still has the agentic files>, e.g. the
       commit just before the last sync."
fi
echo "known-good agentic tree: $KNOWN_GOOD"

# --- merge -------------------------------------------------------------------
# Bring in all of `main`. On the first sync this is a fast-forward that also
# deletes the agentic files; on later syncs it is an ordinary merge.
echo ">>> merging $REMOTE/$SOURCE_BRANCH into $TARGET_BRANCH ..."
git merge --no-edit "$REMOTE/$SOURCE_BRANCH"

# --- restore the agentic layer ----------------------------------------------
# Re-materialize every agentic path from the known-good tree. `git checkout
# <tree> -- <paths>` stages them; paths untouched by the merge are no-ops.
echo ">>> restoring agentic paths from $KNOWN_GOOD ..."
git checkout "$KNOWN_GOOD" -- "${AGENTIC_PATHS[@]}"

if git diff --cached --quiet; then
  echo ">>> agentic layer already intact after merge; nothing to restore."
else
  if [ "$DRY_RUN" -eq 1 ]; then
    echo ">>> --dry-run: would commit the following restored paths:"
    git diff --cached --name-status
    echo ">>> --dry-run: no commit made. Inspect, then re-run without --dry-run."
    exit 0
  fi
  git commit --no-edit \
    --author="KiroCrew <kirocrew@users.noreply.github.com>" \
    -m "Restore agentic layer after syncing $SOURCE_BRANCH

$SOURCE_BRANCH's revert of the agentic workflow deletes the agentic configs,
docs, skills and eval cases on merge. Re-add them from the pre-merge
$TARGET_BRANCH tree ($KNOWN_GOOD) so code and bug fixes from $SOURCE_BRANCH
flow through while the agentic layer is preserved."
fi

# --- verify ------------------------------------------------------------------
echo ""
echo "=== verification ==="
if git diff --quiet "$KNOWN_GOOD" HEAD -- "${AGENTIC_PATHS[@]}"; then
  echo "OK: agentic paths are byte-identical to the known-good tree."
else
  echo "WARNING: agentic paths differ from known-good — inspect:"
  git diff --stat "$KNOWN_GOOD" HEAD -- "${AGENTIC_PATHS[@]}"
fi

echo ""
echo "commits brought in from $SOURCE_BRANCH:"
git log --oneline "$KNOWN_GOOD..HEAD"

echo ""
echo "Done. Review with:  git show HEAD   and   git log --oneline"
echo "Then push:          git push $REMOTE $TARGET_BRANCH"
echo "(or push a feature branch and open a PR if $TARGET_BRANCH is protected)"
