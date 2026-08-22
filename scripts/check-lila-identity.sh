#!/usr/bin/env bash
set -euo pipefail

mapping="docs/rust-rewrite/lila-identity-map.tsv"
failures=0

fail() {
  printf 'check-lila-identity: %s\n' "$*" >&2
  failures=$((failures + 1))
}

if [ ! -f "$mapping" ]; then
  fail "missing identity map: $mapping"
else
  mapping_error="$(awk -F '\t' '
    NR == 1 {
      if ($0 != "order\tmatch_kind\told\tcanonical\tpolicy\tsurface") {
        print "invalid header"
        exit
      }
      next
    }
    NF != 6 {
      print "row " NR " has " NF " fields; expected 6"
      exit
    }
    $1 != sprintf("%03d", NR - 1) {
      print "row " NR " has non-contiguous order " $1
      exit
    }
    END {
      if (NR != 90) print "expected 89 mappings, found " NR - 1
    }
  ' "$mapping")"
  if [ -n "$mapping_error" ]; then
    fail "$mapping_error"
  fi
fi

for required in \
  crates/lila-front \
  crates/lila-ir \
  crates/lila-runtime \
  crates/lila-spec-exec \
  crates/lila-aot-wasm \
  crates/lila-backend-c \
  crates/lila-backend-native \
  crates/lila-engine \
  crates/lila-test262 \
  crates/lila-cli \
  crates/lila-cli/src/bin/lila.rs
do
  [ -e "$required" ] || fail "missing canonical path: $required"
done

old_path_pattern='(^|/)(porffor-[^/]*|porf([.]rs|[.]cmd)?|porffor[.](jsonc|json|toml))($|/)'
old_token_pattern='porffor-|porffor_|PORFFOR_|__porf|\$Porffor|\$porffor\$module\$|porf_host|CARGO_BIN_EXE_porf|unsupported in porffor|not supported in porffor|porffor wasm trace:|(^|[^[:alnum:]_])porf([^[:alnum:]_]|$)'
old_word_pattern='(^|[^[:alnum:]_])(porffor|Porffor)([^[:alnum:]_]|$)'

while IFS= read -r -d '' path; do
  [ -e "$path" ] || continue

  case "$path" in
    vendor/*|test262/vendor/*|test262/snapshots/*) continue ;;
    crates/lila-test262/tests/fixtures/*/snapshots/*) continue ;;
    docs/rust-rewrite/lila-identity-map.tsv) continue ;;
    docs/rust-rewrite/lila-identity-migration.md) continue ;;
    scripts/check-lila-identity.sh) continue ;;
    scripts/check-no-legacy-js.sh) continue ;;
    # The status upgrader must recognize the retired generated-block spelling,
    # and its regression suite must construct that historical input. These are
    # migration readers/tests, not product or publication identities.
    scripts/check-readme-status-artifacts.sh) continue ;;
    scripts/tests/check-readme-status-artifacts.sh) continue ;;
    tasks/28-retire-legacy-js.md|tasks/29-lila-identifier-migration.md) continue ;;
  esac

  if printf '%s\n' "$path" | grep -Eq "$old_path_pattern"; then
    fail "transitional path remains: $path"
  fi

  matches="$(grep -nEI "$old_token_pattern" "$path" 2>/dev/null || true)"
  if [ -n "$matches" ]; then
    while IFS= read -r match; do
      [ -n "$match" ] && fail "$path:$match"
    done <<EOF
$matches
EOF
  fi

  case "$path" in
    *.md|index.html|CNAME) continue ;;
  esac

  matches="$(grep -nEI "$old_word_pattern" "$path" 2>/dev/null \
    | grep -Fv 'https://github.com/mewhhaha/porffor' \
    | grep -Fv 'porffor.dev' || true)"
  if [ -n "$matches" ]; then
    while IFS= read -r match; do
      [ -n "$match" ] && fail "$path:$match"
    done <<EOF
$matches
EOF
  fi
done < <(git ls-files -z --cached --others --exclude-standard)

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-lila-identity: ok\n'
