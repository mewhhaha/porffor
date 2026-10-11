#!/usr/bin/env bash
set -euo pipefail
verification_flags=(--memory-mib 4096)
closure_flags=()
cloud_selected=0
reuse_selected=0
for option in "$@"; do
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
    *)
      echo 'usage: check-test262-closure.sh [--cloud] [--reuse-case-workers]' >&2
      exit 2
      ;;
  esac
done

# Separate from informational status publication. The native owner performs and
# validates both full fresh families; this driver only enforces the resource cap.
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd -- "$repo_root"
closure_binary="${LILA_BIN:-$repo_root/target/release/lila}"
closure_evidence="${CLOSURE_EVIDENCE_DIR:-$repo_root/target/test262-closure}"
if [[ ! -x "$closure_binary" ]]; then
  echo "missing closure executable: $closure_binary" >&2
  exit 1
fi
python3 "$repo_root/scripts/limited_verification.py" "${verification_flags[@]}" -- \
  "$closure_binary" --jobs 1 test262 sync --execution-backend wasm-aot
exec python3 "$repo_root/scripts/limited_verification.py" "${verification_flags[@]}" -- \
  "$closure_binary" --jobs 1 test262 close-release --snapshot-dir "$closure_evidence" "${closure_flags[@]}"
