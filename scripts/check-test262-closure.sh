#!/usr/bin/env bash
set -euo pipefail
verification_flags=(--memory-mib 4096)
closure_flags=()
cloud_selected=0
reuse_selected=0
threads_selected=0
threads=1
while (( $# )); do
  option="$1"
  shift
  case "$option" in
    --cloud)
      [[ "$cloud_selected" == 0 ]] || { echo 'duplicate --cloud' >&2; exit 2; }
      cloud_selected=1
      verification_flags=(--cloud)
      ;;
    --reuse-case-workers)
      [[ "$reuse_selected" == 0 ]] || { echo 'duplicate --reuse-case-workers' >&2; exit 2; }
      reuse_selected=1
      closure_flags+=(--reuse-case-workers)
      ;;
    --threads)
      [[ "$threads_selected" == 0 && $# -gt 0 ]] || { echo 'closure requires --threads once with a positive integer' >&2; exit 2; }
      threads="$1"
      shift
      [[ "$threads" =~ ^[1-9][0-9]{0,17}$ ]] || { echo 'closure --threads must be a positive decimal integer' >&2; exit 2; }
      threads_selected=1
      closure_flags+=(--threads "$threads")
      ;;
    *)
      echo 'usage: check-test262-closure.sh [--cloud] [--threads N] [--reuse-case-workers]' >&2
      exit 2
      ;;
  esac
done
if [[ "$threads" != 1 && "$cloud_selected" != 1 ]]; then
  echo 'parallel closure requires explicit --cloud; local verification stays one worker' >&2
  exit 2
fi
closure_verification_flags=("${verification_flags[@]}")
if [[ "$threads" != 1 ]]; then
  closure_verification_flags+=(--cloud-worker-cpus auto)
fi

# Separate from informational status publication. The native owner performs and
# validates both full fresh families; this driver only enforces the resource cap.
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd -- "$repo_root"
closure_binary="${SCRIPTW_BIN:-$repo_root/target/release/scriptw}"
closure_evidence="${CLOSURE_EVIDENCE_DIR:-$repo_root/target/test262-closure}"
if [[ ! -x "$closure_binary" ]]; then
  echo "missing closure executable: $closure_binary" >&2
  exit 1
fi
python3 "$repo_root/scripts/limited_verification.py" "${verification_flags[@]}" -- \
  "$closure_binary" --jobs 1 test262 sync --execution-backend wasm-aot
exec python3 "$repo_root/scripts/limited_verification.py" "${closure_verification_flags[@]}" -- \
  "$closure_binary" --jobs 1 test262 close-release --snapshot-dir "$closure_evidence" "${closure_flags[@]}"
