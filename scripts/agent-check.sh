#!/usr/bin/env bash
# agent-check.sh — the calibrated pre-change gate for `tc`.
#
# Runs the checks an agent's change must pass, calibrated to the current HEAD
# baseline (see docs/agents/CALIBRATION.md). It is intentionally NOT `-e`: it runs
# every check, collects failures, and reports them together.
#
# Design note: HEAD is NOT globally fmt-clean and NOT globally clippy-clean (the
# `compiler` crate even has 2 pre-existing error-level clippy lints). A gate that
# fails on that pre-existing debt would just train agents to fight the codebase, so
# fmt and clippy are DIFF-SCOPED: they judge only the files the change touched.
#
# Usage:
#   ./scripts/agent-check.sh            # compare working tree against $BASE (default origin/main)
#   BASE=main ./scripts/agent-check.sh  # override the base ref
#
# Requires: stable rustc >= 1.95, nightly (for fmt), python3. If cargo is not on
# PATH, add "$HOME/.cargo/bin" first.

set -uo pipefail

BASE="${BASE:-origin/main}"
git rev-parse --verify "$BASE" >/dev/null 2>&1 || BASE="main"
FAIL=0
note()  { printf '\n\033[1m== %s ==\033[0m\n' "$1"; }
bad()   { printf '\033[31mFAIL:\033[0m %s\n' "$1"; FAIL=1; }
warn()  { printf '\033[33mwarn:\033[0m %s\n' "$1"; }
ok()    { printf '\033[32mok:\033[0m %s\n' "$1"; }

changed() { { git diff --name-only --diff-filter=ACM "$BASE"...HEAD -- "$@" 2>/dev/null; \
              git diff --name-only --diff-filter=ACM -- "$@"; } | sort -u | sed '/^$/d'; }
CHANGED_RS=$(changed '*.rs')
CHANGED_TOML=$(changed '*/Cargo.toml' Cargo.toml)

# --- 1. fmt (nightly, diff-scoped) ---
note "rustfmt (nightly, changed files only)"
if [ -z "$CHANGED_RS" ]; then
  ok "no changed .rs files"
elif rustfmt +nightly --version >/dev/null 2>&1; then
  if printf '%s\n' "$CHANGED_RS" | xargs rustfmt +nightly --edition 2024 --check --config-path ./rustfmt.toml >/tmp/agent-fmt.log 2>&1; then
    ok "changed files are nightly-fmt clean"
  else
    bad "rustfmt drift in changed files (/tmp/agent-fmt.log). Run: cargo +nightly fmt"
  fi
else
  warn "nightly rustfmt not installed — run: rustup toolchain install nightly --component rustfmt"
fi

# --- 2. clippy (LINE-scoped): fail only on findings on lines this change added ---
# HEAD is not clippy-clean (deliberate idioms + 2 pre-existing correctness lints in
# `compiler`). --cap-lints=warn lets clippy analyze the whole workspace anyway; the
# python filter then keeps only findings whose primary span is on a line THIS branch
# added/changed (computed from the merge-base diff). Line-scoping — not file-scoping —
# is what stops a change from being blamed for a touched file's pre-existing idioms.
note "clippy (workspace, findings scoped to changed lines)"
if [ -z "$CHANGED_RS" ]; then
  if cargo clippy --workspace -- --cap-lints=warn >/tmp/agent-clippy.log 2>&1; then
    ok "workspace lints/compiles (no changed .rs to scope)"
  else
    bad "cargo clippy failed to run (see /tmp/agent-clippy.log)"
  fi
else
  MB=$(git merge-base "$BASE" HEAD 2>/dev/null || echo "$BASE")
  git diff --unified=0 "$MB" -- '*.rs' > /tmp/agent-added.patch 2>/dev/null
  cargo clippy --workspace --message-format=json -- --cap-lints=warn >/tmp/agent-clippy.json 2>/tmp/agent-clippy.err
  python3 - <<'PY'
import json, re, sys
# 1) added/changed lines per file, from the merge-base diff
added = {}
cur = None
hunk = re.compile(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@')
for ln in open("/tmp/agent-added.patch", encoding="utf-8", errors="replace"):
    if ln.startswith("+++ "):
        p = ln[4:].strip()
        cur = None if p == "/dev/null" else re.sub(r'^[ab]/', '', p)
    elif ln.startswith("@@") and cur:
        m = hunk.match(ln)
        if m:
            start = int(m.group(1)); cnt = int(m.group(2) or "1")
            added.setdefault(cur, set()).update(range(start, start + max(cnt, 1)))
# 2) clippy findings whose primary span line is an added line
hits = []
for line in open("/tmp/agent-clippy.json", encoding="utf-8", errors="replace"):
    line = line.strip()
    if not line.startswith("{"):
        continue
    try: obj = json.loads(line)
    except Exception: continue
    if obj.get("reason") != "compiler-message": continue
    msg = obj.get("message") or {}
    if msg.get("level") not in ("warning", "error"): continue
    code = ((msg.get("code") or {}).get("code")) or msg["level"]
    for sp in msg.get("spans", []):
        if not sp.get("is_primary"): continue
        f = sp.get("file_name"); 
        for l in range(sp.get("line_start", 0), sp.get("line_end", sp.get("line_start", 0)) + 1):
            if l in added.get(f, ()):
                hits.append(f"{f}:{sp['line_start']}: [{code}] {msg.get('message','')}")
                break
        break
if hits:
    print("\n".join(sorted(set(hits)))); sys.exit(1)
sys.exit(0)
PY
  if [ $? -eq 0 ]; then ok "no new clippy findings on changed lines"; else bad "clippy findings on changed lines (fix or justify)"; fi
fi

# --- 3. build + the tests CI actually runs (workspace tests do not compile yet) ---
note "cargo build"
if cargo build >/tmp/agent-build.log 2>&1; then ok "builds"; else bad "cargo build failed (/tmp/agent-build.log)"; fi
note "cargo test -p composer"
if cargo test -p composer >/tmp/agent-test.log 2>&1; then ok "composer tests pass"; else bad "composer tests failed (/tmp/agent-test.log)"; fi

# --- 4. convention greps on changed files ---
note "convention checks (changed files)"
if [ -n "$CHANGED_TOML" ] && grep -nE '^\s*(anyhow|thiserror)\s*=' $CHANGED_TOML >/dev/null 2>&1; then
  bad "anyhow/thiserror added to a Cargo.toml — the codebase deliberately avoids them"
else ok "no anyhow/thiserror dependency added"; fi
if [ -n "$CHANGED_RS" ] && grep -nE '^\s*use\s+(anyhow|thiserror)\b' $CHANGED_RS >/dev/null 2>&1; then
  bad "use anyhow/thiserror in changed code — use the fail-fast posture instead"
else ok "no anyhow/thiserror imports"; fi
if [ -n "$CHANGED_RS" ] && grep -nE '\bjoin_all\s*\(' $CHANGED_RS >/dev/null 2>&1; then
  bad "unbounded join_all — use futures::stream…buffer_unordered(n) with a clamped cap"
else ok "no unbounded join_all"; fi
if [ -n "$CHANGED_RS" ] && grep -nE '"" *\. *to_string\(\)|String::new\(\)' $CHANGED_RS >/dev/null 2>&1; then
  warn "empty-string literal in changed code — prefer kit::empty()"
fi

note "result"
if [ "$FAIL" -eq 0 ]; then ok "agent-check passed"; else bad "agent-check FAILED — fix the items above"; fi
exit "$FAIL"
