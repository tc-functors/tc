#!/usr/bin/env bash
#
# lint-boundaries.sh — Enforces architectural boundary invariants.
#
# Checks:
#   1. Filesystem operations only in compiler/kit
#   2. Inference logic only in composer
#   3. No duplicate function definitions across boundaries
#
# Exit code: number of violations found (0 = clean)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

VIOLATIONS=0

header() {
    echo ""
    echo -e "${YELLOW}━━━ $1 ━━━${NC}"
}

pass() {
    echo -e "  ${GREEN}✓${NC} $1"
}

fail() {
    echo -e "  ${RED}✗${NC} $1"
    VIOLATIONS=$((VIOLATIONS + 1))
}

# ─────────────────────────────────────────────────────────────────────────────
# Check 1: Filesystem boundary
# ─────────────────────────────────────────────────────────────────────────────
header "Filesystem boundary (only compiler/kit may do FS I/O)"

FS_PATTERNS=(
    'std::fs::'
    'File::open'
    'File::create'
    'read_to_string'
    'create_dir_all'
    'fs::metadata'
    'fs::read_dir'
    'use walkdir'
    'WalkDir::new'
)

ALLOWED_FS="lib/compiler|lib/kit"

for pattern in "${FS_PATTERNS[@]}"; do
    matches=$(rg -l --glob '*.rs' "$pattern" lib/ 2>/dev/null | grep -vE "$ALLOWED_FS" || true)
    if [ -n "$matches" ]; then
        while IFS= read -r file; do
            fail "$file uses '$pattern'"
        done <<< "$matches"
    fi
done

if [ $VIOLATIONS -eq 0 ]; then
    pass "No filesystem violations"
fi

FS_VIOLATIONS=$VIOLATIONS

# ─────────────────────────────────────────────────────────────────────────────
# Check 2: Inference boundary
# ─────────────────────────────────────────────────────────────────────────────
header "Inference boundary (only composer may define inference logic)"

INFERENCE_PATTERNS=(
    'fn infer_'
    'fn guess_'
    'fn discover_'
    'fn find_implicit_'
    'fn is_inferred_'
)

ALLOWED_INFERENCE="lib/composer"

for pattern in "${INFERENCE_PATTERNS[@]}"; do
    matches=$(rg -l --glob '*.rs' "$pattern" lib/ 2>/dev/null | grep -vE "$ALLOWED_INFERENCE" || true)
    if [ -n "$matches" ]; then
        while IFS= read -r file; do
            # Show the actual function signature for clarity
            sigs=$(rg -n "$pattern" "$file" 2>/dev/null | head -3)
            fail "$file defines inference function:\n        $sigs"
        done <<< "$matches"
    fi
done

INFERENCE_VIOLATIONS=$((VIOLATIONS - FS_VIOLATIONS))
if [ $INFERENCE_VIOLATIONS -eq 0 ]; then
    pass "No inference boundary violations"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Check 3: No duplicate public function definitions across crate boundaries
# ─────────────────────────────────────────────────────────────────────────────
header "Duplicate definitions (same pub fn in multiple crates)"

KNOWN_DUPLICATES=(
    "infer_lang"
    "guess_runtime"
    "is_topology_dir"
)

PRE_DUP_VIOLATIONS=$VIOLATIONS

for fn_name in "${KNOWN_DUPLICATES[@]}"; do
    crates=$(rg -l --glob '*.rs' "pub fn $fn_name" lib/ 2>/dev/null | sed 's|lib/\([^/]*\)/.*|\1|' | sort -u)
    count=$(echo "$crates" | grep -c . || true)
    if [ "$count" -gt 1 ]; then
        crate_list=$(echo "$crates" | tr '\n' ', ' | sed 's/,$//')
        fail "'$fn_name' defined in $count crates: $crate_list"
    fi
done

DUP_VIOLATIONS=$((VIOLATIONS - PRE_DUP_VIOLATIONS))
if [ $DUP_VIOLATIONS -eq 0 ]; then
    pass "No duplicate definitions"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Check 4: Coding style (lifetimes, error types, traits)
# ─────────────────────────────────────────────────────────────────────────────
header "Coding style (avoid lifetimes, custom error types, custom traits)"

PRE_STYLE_VIOLATIONS=$VIOLATIONS

# Lifetime annotations in struct/enum/fn definitions
# Allowed: lisp/parser.rs (nom), differ/src/lib.rs (tree borrows), kit/src/core.rs (slice output)
lifetime_hits=$(rg -n "(struct|enum|fn)\s+\w+<'" lib/ --glob '*.rs' 2>/dev/null \
    | grep -v "lisp/parser.rs" \
    | grep -v "differ/src/lib.rs" \
    | grep -v "kit/src/core.rs" || true)
if [ -n "$lifetime_hits" ]; then
    while IFS= read -r hit; do
        fail "Explicit lifetime annotation: $hit"
    done <<< "$lifetime_hits"
fi

# Custom error types (allow ParseError as grandfathered)
error_type_hits=$(rg -n "(enum|struct)\s+\w*Error" lib/ --glob '*.rs' 2>/dev/null | grep -v "ParseError" || true)
if [ -n "$error_type_hits" ]; then
    while IFS= read -r hit; do
        fail "Custom error type: $hit"
    done <<< "$error_type_hits"
fi

# Custom trait definitions
trait_hits=$(rg -n "^pub trait |^trait " lib/ --glob '*.rs' 2>/dev/null || true)
if [ -n "$trait_hits" ]; then
    while IFS= read -r hit; do
        fail "Custom trait definition: $hit"
    done <<< "$trait_hits"
fi

STYLE_VIOLATIONS=$((VIOLATIONS - PRE_STYLE_VIOLATIONS))
if [ $STYLE_VIOLATIONS -eq 0 ]; then
    pass "No coding style violations"
fi

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}All boundary checks passed.${NC}"
else
    echo -e "${RED}$VIOLATIONS boundary violation(s) found.${NC}"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

exit $VIOLATIONS
