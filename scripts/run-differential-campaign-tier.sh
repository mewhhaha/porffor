#!/usr/bin/env bash
# Existing selected workers own execution; every grammar retains its red evidence.
set -euo pipefail
verification_flags=(--memory-mib 4096)
if [[ ${1:-} == --cloud ]]; then
  verification_flags=(--cloud)
  shift
fi
if [[ $# != 3 ]]; then
  echo 'usage: run-differential-campaign-tier.sh [--cloud] CLI OUTPUT_DIRECTORY pr-fast|nightly' >&2
  exit 2
fi
campaign_cli=$1
campaign_root=$2
case $3 in
  pr-fast) campaign_cases=2; campaign_replays=16 ;;
  nightly) campaign_cases=64; campaign_replays=64 ;;
  *) echo 'campaign tier must be pr-fast or nightly' >&2; exit 2 ;;
esac
if [[ ! -x $campaign_cli ]]; then
  echo 'the selected feature-enabled CLI executable is required' >&2
  exit 2
fi
mkdir -- "$campaign_root"
campaign_failed=0
run_grammar() {
  local campaign_grammar=$1
  local campaign_seed=$2
  shift 2
  if ! python3 scripts/limited_verification.py "${verification_flags[@]}" -- \
    "$campaign_cli" --jobs 1 differential campaign \
    --output-dir "$campaign_root/$campaign_grammar" --seed "$campaign_seed" --cases "$campaign_cases" \
    --max-replays "$campaign_replays" --oracle spec-exec --grammar "$campaign_grammar" "$@"; then
    campaign_failed=1
  fi
}
run_grammar integer-arithmetic-v1 1 --checks 4 --depth 2
run_grammar integer-bitwise-v2 1 --checks 4 --depth 2
run_grammar integer-product-v3 1 --checks 4 --depth 2
run_grammar object-probe-v1 1 --nodes 4 --properties 6
run_grammar object-mutations-v2 1 --nodes 4 --properties 6 --steps 16
run_grammar module-graph-v1 1 --modules 4 --edges 6
run_grammar module-graph-v2 1 --modules 4 --edges 6
run_grammar control-flow-v1 1 --steps 8 --depth 4
run_grammar negative-source-v1 1 --steps 8 --depth 4
run_grammar builtin-stateful-v2 6 --steps 8
run_grammar metamorphic-stateful-v2 6 --steps 8
exit "$campaign_failed"
