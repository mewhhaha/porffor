#!/usr/bin/env bash
set -euo pipefail

mapping="docs/rust-rewrite/scriptw-identity-map.tsv"
failures=0

fail() {
  printf 'check-scriptw-identity: %s\n' "$*" >&2
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
  crates/scriptw-front \
  crates/scriptw-ir \
  crates/scriptw-runtime \
  crates/scriptw-runtime-image \
  crates/scriptw-intl \
  crates/scriptw-intl-collator-data \
  crates/scriptw-verification-feature-roots \
  crates/scriptw-spec-exec \
  crates/scriptw-aot-wasm \
  crates/scriptw-engine \
  crates/scriptw-test262 \
  crates/scriptw-cli \
  crates/scriptw-cli/src/bin/scriptw.rs
do
  [ -e "$required" ] || fail "missing canonical path: $required"
done

old_path_pattern='(^|/)((porffor|lila)-[^/]*|lila([.]rs)?|porf([.]rs|[.]cmd)?|porffor[.](jsonc|json|toml))($|/)'
old_token_pattern='porffor-|porffor_|PORFFOR_|__porf|\$Porffor|\$porffor\$module\$|porf_host|CARGO_BIN_EXE_porf|unsupported in porffor|not supported in porffor|porffor wasm trace:|(^|[^[:alnum:]_])porf([^[:alnum:]_]|$)'
old_word_pattern='(^|[^[:alnum:]_])(porffor|Porffor)([^[:alnum:]_]|$)'
old_token_pattern="$old_token_pattern"'|lila-|lila_|LILA_|__lila|liblila|CARGO_BIN_EXE_lila|\$Lila|CurrentLila|(^|[^[:alnum:]_])lila([^[:alnum:]_]|$)'

# Preserve the GNU grep expressions while batching files and admitting frozen
# provenance in one Python process. Exclusions are applied before filesystem work.
scanner="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/identity_scan.py"
if ! python3 -B "$scanner" "$old_path_pattern" "$old_token_pattern" "$old_word_pattern"; then
  failures=$((failures + 1))
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-scriptw-identity: ok\n'
